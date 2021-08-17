use std::path::Path;
use std::sync::{Arc, Mutex, atomic::{AtomicU32, AtomicU64, Ordering}};
use wasmer::{
    imports, Function, Instance, LazyInit, Memory, MemoryView, Module, Store, Value, WasmerEnv,
};
use shared::{from_i64, from_i32, from_u64, from_u32};

#[derive(WasmerEnv, Clone, Debug)]
struct Runtime {
    inner: Arc<InnerRuntime>,
    #[wasmer(export)]
    memory: LazyInit<Memory>,
}

#[derive(Debug)]
struct InnerRuntime {
    store: Mutex<Vec<Vec<u8>>>,
    return_value: Mutex<Vec<u8>>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            inner: Arc::new(InnerRuntime {
                store: Default::default(),
                return_value: Default::default(),
            }),
            memory: LazyInit::new(),
        }
    }

    pub fn read_slice(&self, ptr: i64, len: i32) -> Vec<u8> {
        println!("read_slice");
        let view: MemoryView<u8> = self.memory.get_ref().expect("missing memory").view();
        let ptr = from_i64(ptr);
        let len = from_i32(len);
        view[ptr as usize..ptr as usize + len as usize].iter().map(|v| v.get()).collect()
    }
    
    pub fn pop_return<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        let mut lock = self.inner.return_value.lock().unwrap();
        let res = serde_cbor::from_slice(&lock[..]).unwrap();
        lock.clear();
        res
    }
}

pub struct Vm {
    env: Runtime,
    instance: Instance,
    mm: ManagedMemory,
}

impl Vm {
    pub fn new(source: impl AsRef<Path>) -> anyhow::Result<Self> {
        println!("Starting...");
        let store = Store::default();
        let module = Module::from_file(&store, source)?;

        let mm = ManagedMemory::default();
        let env = Runtime::new();

        fn rt_create(env: &Runtime, ptr: i64, len: i32) {
            println!("rt_crate");
            let s = env.read_slice(ptr, len);
            env.inner.store.lock().unwrap().push(s);
        }

        fn write_return_bytes(env: &Runtime, ptr: i64, len: i32) {
            println!("return bytes");
            *env.inner.return_value.lock().unwrap() = env.read_slice(ptr, len);
        }


        fn dbg(a: i32) {
            println!("WASM DEBUG: {}", a);
        }

        let import_object = imports! {
            "env" => {
                "rt_create" => Function::new_native_with_env(&store, env.clone(), rt_create),
                "write_return_bytes" => Function::new_native_with_env(&store, env.clone(), write_return_bytes),
                "dbg" => Function::new_native(&store, dbg),
            }
        };
        let instance = Instance::new(&module, &import_object)?;

        Ok(Vm {
            env,
            instance,
            mm,
        })
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let ret: shared::NewReturn = self.call("cool_actor_constructor", shared::NewParams {})?;
        dbg!(ret);
        Ok(())
    }

    fn call<IN: serde::Serialize, OUT: serde::de::DeserializeOwned>(&self, method: &str, args: IN) -> anyhow::Result<OUT> {
        println!("calling {}", method);
        let fun = self.instance.exports.get_function(method)?;

        let (ptr, len) = self.write_args(args);
        
        let _result = fun.call(&[Value::I64(from_u64(ptr)), Value::I32(from_u32(len))])?;

        let out = self.read_return();
        
        Ok(out)
    }

    fn write_args<T: serde::Serialize>(&self, args: T) -> (u64, u32) {
        let bytes = serde_cbor::to_vec(&args).unwrap();
        self.write_bytes(dbg!(&bytes))
    }
    
    fn write_bytes(&self, bytes: &[u8]) -> (u64, u32) {
        let len = bytes.len();
        let ptr = self.alloc(len as u32 + 8);
        dbg!(len, ptr);
        
        self.memory().view()[ptr as usize..ptr as usize + len + 8]
            .iter()
            .zip((len as u64).to_le_bytes().iter().chain(bytes.iter()))
            .for_each(|(cell, byte)| cell.set(*byte));

        (ptr as u64, len as u32)
    }

    fn alloc(&self, size: u32) -> u64 {
        self.mm.alloc(size, self.allocator())
    }
    
    fn memory(&self) -> &Memory {
        self.instance.exports.get_memory("memory").unwrap()
    }

    fn allocator(&self) -> &Function {
        self.instance.exports.get_function("alloc_buffer").unwrap()
    }

    fn read_return<T: serde::de::DeserializeOwned>(&self) -> T {
        self.env.pop_return().unwrap()
    }

    fn read_bytes(&self, ptr: u64, len: u32) -> Vec<u8> {
        self.memory().view()[(ptr as usize)..(ptr as usize) + len as usize]
            .iter()
            .map(|x| x.get())
            .collect()
    }
}


// TODO: Atomics
#[derive(Default, Debug)]
struct ManagedMemory {
    ptr: AtomicU64,
    len: AtomicU32,
}


impl ManagedMemory {
    pub fn alloc(&self, size: u32, allocator: &Function) -> u64 {
        if self.len.load(Ordering::SeqCst) >= size {
            return self.ptr.load(Ordering::SeqCst);
        }

        let result = allocator.call(&[Value::I32(from_u32(size))]).unwrap();
        let ptr = from_i64(result[0].unwrap_i64());
        self.ptr.store(ptr, Ordering::SeqCst);
        self.len.store(size, Ordering::SeqCst);
        ptr
    }
}
