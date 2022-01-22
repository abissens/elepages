use crate::config::Value;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait Printer {
    fn print(&self, caller: &str, message: &str);
}

#[allow(clippy::upper_case_acronyms)]
pub enum PrintLevel {
    V,
    VV,
    VVV,
}

pub const PRINT_LEVEL_V: PrintLevel = PrintLevel::V;
pub const PRINT_LEVEL_VV: PrintLevel = PrintLevel::VV;
pub const PRINT_LEVEL_VVV: PrintLevel = PrintLevel::VVV;

impl PrintLevel {
    pub(crate) fn should_print(&self, print_level: &PrintLevel) -> bool {
        match self {
            PrintLevel::V => match print_level {
                PrintLevel::V => true,
                PrintLevel::VV => false,
                PrintLevel::VVV => false,
            },
            PrintLevel::VV => match print_level {
                PrintLevel::V => true,
                PrintLevel::VV => true,
                PrintLevel::VVV => false,
            },
            PrintLevel::VVV => true,
        }
    }
}
pub struct Env {
    values: Arc<Mutex<HashMap<String, Value>>>,
    printer: Box<dyn Printer + Send + Sync>,
    print_level: Option<PrintLevel>,
}

impl Env {
    pub fn can_print(&self, level: &PrintLevel) -> bool {
        if let Some(print_level) = &self.print_level {
            return print_level.should_print(level);
        }
        false
    }

    pub fn print(&self, level: &PrintLevel, caller: &str, message: &str) {
        if self.can_print(level) {
            self.printer.print(caller, message);
        }
    }

    pub fn print_v(&self, caller: &str, message: &str) {
        self.print(&PRINT_LEVEL_V, caller, message)
    }

    pub fn print_vv(&self, caller: &str, message: &str) {
        self.print(&PRINT_LEVEL_VV, caller, message)
    }

    pub fn print_vvv(&self, caller: &str, message: &str) {
        self.print(&PRINT_LEVEL_VVV, caller, message)
    }

    pub fn new(printer: Box<dyn Printer + Send + Sync>, print_level: Option<PrintLevel>) -> Self {
        Self {
            values: Default::default(),
            printer,
            print_level,
        }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.values.lock().unwrap().get(key).cloned()
    }

    pub fn insert(&self, key: String, value: Value) {
        self.values.lock().unwrap().insert(key, value);
    }
}

impl Env {
    pub fn test() -> Self {
        Self::new(Box::new(NoopPrinter), None)
    }

    pub fn default_for_level(level: Option<PrintLevel>) -> Self {
        Self::new(Box::new(DefaultPrinter), level)
    }
}

impl Default for Env {
    fn default() -> Self {
        Env::default_for_level(Some(PrintLevel::V))
    }
}

pub struct DefaultPrinter;

impl Printer for DefaultPrinter {
    fn print(&self, caller: &str, message: &str) {
        println!("{} [{}] {}", Utc::now().format("%Y-%b-%d %T"), caller, message)
    }
}

struct NoopPrinter;
impl Printer for NoopPrinter {
    fn print(&self, _: &str, _: &str) {}
}
