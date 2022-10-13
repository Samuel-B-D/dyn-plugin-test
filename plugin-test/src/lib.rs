use std::{any::Any, pin::Pin, fmt::Debug};
use anyhow::anyhow;
use once_cell::sync::OnceCell;
#[cfg(feature="dynamic")]
use libloading::{Library, Symbol};

pub trait Handler: Any + Debug {
    fn on_init(&mut self) -> anyhow::Result<()>;
    fn on_something(&mut self) -> anyhow::Result<()>;
    fn on_drop(&mut self);
}

pub type HandlerConstructor = unsafe fn() -> *mut HandlerWrapper;

pub struct HandlerWrapper(pub *mut dyn Handler);

#[macro_export]
macro_rules! declare_handler {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _create_handler() -> *mut $crate::HandlerWrapper {
            // make sure the constructor is the correct type.
            let constructor: unsafe fn() -> $plugin_type = $constructor;

            let boxed_handler = unsafe { Box::new(constructor()) };
            println!("Created Handler at: {:p}", boxed_handler);
            let handler_ptr = Box::into_raw(boxed_handler);
            println!("Raw handler ptr at: {:p}", handler_ptr);

            let wrapper = Box::new($crate::HandlerWrapper(handler_ptr));
            Box::into_raw(wrapper)

            // handler_ptr
        }
    };
}

#[derive(Debug)]
pub struct SomeHandler {
    data: OnceCell<Vec<i32>>,
}

impl SomeHandler {
    pub fn uninit() -> Self {
        Self { data: OnceCell::new() }
    }
}

impl Handler for SomeHandler {
    fn on_init(&mut self) -> anyhow::Result<()> {
        self.data.set(vec![1,2,3]).map_err(|_| anyhow!("Unable to init OnceCell"))?;
        println!("Handler initialized");
        Ok(())
    }

    fn on_something(&mut self) -> anyhow::Result<()> {
        println!("on_something");
        Ok(())
    }

    fn on_drop(&mut self) {
        println!("Handler dropped");
    }
}

pub struct App {
    #[cfg(feature="dynamic")]
    loader: Option<Library>,
    handler: Option<Pin<Box<dyn Handler>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            #[cfg(feature="dynamic")]
            loader: None,
            handler: None,
        }
    }

    unsafe fn handler_mut(&mut self) -> anyhow::Result<&mut dyn Handler> {
        Ok(self.handler.as_mut().ok_or(anyhow!("Handler not properly loaded"))?.as_mut().get_unchecked_mut())
    }

    pub fn load_handler(&mut self) -> anyhow::Result<()> {
        #[cfg(feature="dynamic")]
        self.load_dynamic_handler()?;

        #[cfg(not(feature="dynamic"))]
        self.load_embedded_handler()?;

        unsafe { self.handler_mut()? }.on_init()?;

        Ok(())
    }

    #[cfg(not(feature="dynamic"))]
    fn load_embedded_handler(&mut self) -> anyhow::Result<()> {
        let handler: Pin<Box<dyn Handler>> = Box::pin(SomeHandler::uninit());
        self.handler = Some(handler);
        Ok(())
    }

    #[cfg(feature="dynamic")]
    fn load_dynamic_handler(&mut self) -> anyhow::Result<()> {
        if self.loader.is_none() {
            let loader = unsafe { Library::new("dyn_plugin_test.dll")? };
            self.loader = Some(loader);
        }
        let loader = self.loader.as_mut().expect("Was just initialized. This is a bug.");
        let handler_constructor: Symbol<HandlerConstructor> = unsafe { loader.get(b"_create_handler")? };
        println!("Loaded handler constructor");
        let handler_wrapper = unsafe { Box::from_raw(handler_constructor()) };
        let handler_ptr = handler_wrapper.0;
        println!("Dynamically created handler at: {:p}", handler_ptr);
        // let handler_ptr = unsafe { handler_constructor() };
        // println!("Dynamically created handler at: {:p}", handler_ptr);
        let handler = unsafe { Box::into_pin(Box::from_raw(handler_ptr)) };
        println!("Re-boxed handler at: {:p}", handler);
        self.handler = Some(handler);
        Ok(())
    }

    pub fn do_something(&mut self) -> anyhow::Result<()> {
        unsafe { self.handler_mut()? }.on_something()?;
        Ok(())
    }

    pub fn unload_handler(&mut self) -> anyhow::Result<()> {
        if self.handler.is_some() {
            unsafe { self.handler_mut()?.on_drop(); }
            self.handler = None;
            #[cfg(feature="dynamic")]
            if let Some(loader) = std::mem::take(&mut self.loader) {
                loader.close()?;
            }
        }
        Ok(())
    }
}