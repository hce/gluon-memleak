#[macro_use]
extern crate gluon_vm;
#[macro_use]
extern crate gluon_codegen;
extern crate gluon;

use std::sync::{ Arc, Mutex };

use gluon::*;
use gluon::import::add_extern_module;
use gluon::vm::api::{ FunctionRef, OwnedFunction };

#[derive(Debug, Userdata, VmType, Clone)]
#[gluon(vm_type = "demo.Context")]
struct Context {
    callbacks: Arc<Mutex<Vec<Callback>>>,
}

#[derive(Debug, Userdata, VmType)]
#[gluon(vm_type = "demo.Callback")]
struct Callback {
    callback: OwnedFunction<fn (Context, String) -> ()>,
    regex: String
}

fn do_it() {
    let bytes = include_bytes!("demo.glu");
    let user_script = String::from_utf8_lossy(bytes);
    fn gluon_register_callback (
        context: &Context,
        regex: String,
        callback: OwnedFunction<fn (Context, String) -> ()>) {
        let callback_request = Callback {
            callback, regex
        };
        context.callbacks.lock().unwrap().push(callback_request);
    }

    fn add_gluon_demo(vm: &Thread) -> vm::Result<vm::ExternModule> {
        vm.register_type::<Context>("demo.Context", &[])?;
        vm.register_type::<Callback>("demo.Callback", &[])?;
        vm::ExternModule::new(vm,record! {
            register_callback => primitive!(3, gluon_register_callback),
            type Context => Context,
            type Callback => Callback
        })
    }

    let vm = new_vm();
    add_extern_module(&vm, "demo", add_gluon_demo);
    let mut script_init: FunctionRef<fn(Context) -> ()>;

    if let Err(e) = Compiler::new()
        .load_script(&vm, "user", &user_script) {
        eprintln!("Script error: {}", e);
        return;
    }
    script_init = vm.get_global("user.init").unwrap();

    let context = Context {
        callbacks: Arc::new(Mutex::new(Vec::new()))
    };

    if let Err(e) = script_init.call(context.clone()) {
        eprintln!("Running script error: {}", e);
    }

    for _i in 0..1000 {
        let mut mr = context.callbacks.lock().unwrap();
        for j in 0 .. mr.len() {
            let mut callback = mr.get_mut(j).unwrap();
            callback.callback.call(context.clone(), callback.regex.clone());
        }
    }

    // Temporary mitigation
    //context.callbacks.lock().unwrap().clear();

}

fn main() {
    loop {
        do_it();
    }
}
