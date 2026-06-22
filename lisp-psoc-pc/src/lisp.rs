use core::fmt::{self, Write};

const MAX_OBJECTS: usize = 384;
const MAX_SYMBOLS: usize = 176;
const MAX_GLOBALS: usize = 104;
const MAX_SYMBOL_BYTES: usize = 32;
const MAX_CALL_ARGS: usize = 16;
const MAX_EVAL_DEPTH: u8 = 128;

type ObjectId = u16;
type SymbolId = u16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LedAction {
    On,
    Off,
    Toggle,
    Status,
}

#[derive(Clone, Copy)]
pub struct Error {
    message: &'static str,
}

impl Error {
    pub const fn new(message: &'static str) -> Self {
        Self { message }
    }

    pub fn message(self) -> &'static str {
        self.message
    }
}

pub trait Board {
    fn led(&mut self, action: LedAction) -> bool;
    fn heartbeat(&mut self, enabled: bool) -> bool;
    fn button_pressed(&mut self, index: i32) -> Result<bool, Error>;
    fn millis(&mut self) -> u32;
    fn read32(&mut self, address: u32) -> Result<u32, Error>;
    fn write32(&mut self, address: u32, value: u32) -> Result<(), Error>;
    fn registers(&mut self) -> RegisterReport;
    fn sd_status(&mut self) -> SdStatusReport;
    fn sd_pins(&mut self) -> SdPinsReport;
    fn sd_pinmux(&mut self) -> SdPinsReport;
    fn sd_clock(&mut self) -> SdClockReport;
    fn sd_init(&mut self) -> SdInitReport;
    fn sd_read0(&mut self) -> SdRead0Report;
    fn sdhc_registers(&mut self) -> SdhcReport;
    fn reboot(&mut self) -> !;
}

#[derive(Clone, Copy)]
pub struct RegisterReport {
    pub scb5_ctrl: u32,
    pub scb5_uart_ctrl: u32,
    pub scb5_rx_status: u32,
    pub scb5_tx_status: u32,
    pub peri_clock5: u32,
    pub peri_div8_0: u32,
    pub hsiom_prt5_sel0: u32,
    pub gpio_prt5_cfg: u32,
    pub gpio_prt13_out: u32,
    pub gpio_prt13_cfg: u32,
}

#[derive(Clone, Copy)]
pub struct SdStatusReport {
    pub cd_low: bool,
    pub prt13_in: u32,
    pub prt13_cfg: u32,
}

#[derive(Clone, Copy)]
pub struct SdPinsReport {
    pub p12_sel1: u32,
    pub p13_sel0: u32,
    pub p12_cfg: u32,
    pub p13_cfg: u32,
}

#[derive(Clone, Copy)]
pub struct SdhcCoreReport {
    pub wrap_ctl: u32,
    pub host_version: u16,
    pub cap1: u32,
    pub cap2: u32,
    pub pstate: u32,
}

#[derive(Clone, Copy)]
pub struct SdhcReport {
    pub sdhc0: SdhcCoreReport,
    pub sdhc1: SdhcCoreReport,
    pub pins: SdPinsReport,
}

#[derive(Clone, Copy)]
pub struct SdClockReport {
    pub path0: u32,
    pub root0: u32,
    pub root2: u32,
    pub fll_config: u32,
    pub fll_config2: u32,
    pub fll_status: u32,
    pub selected_hf_hz: u32,
}

#[derive(Clone, Copy)]
pub struct SdCommandErrorReport {
    pub code: &'static [u8],
    pub normal_int: u16,
    pub error_int: u16,
    pub pstate: u32,
    pub command: u16,
    pub argument: u32,
    pub pstate_after_write: u32,
    pub normal_int_after_write: u16,
    pub error_int_after_write: u16,
}

#[derive(Clone, Copy)]
pub struct SdInitReport {
    pub status: &'static [u8],
    pub cmd8_response: u32,
    pub cmd8_error: Option<SdCommandErrorReport>,
    pub acmd41_ocr: u32,
    pub acmd41_attempts: u16,
    pub gp_out: u32,
    pub gp_in: u32,
    pub host_ctrl1: u8,
    pub host_ctrl2: u16,
    pub xfer_mode: u16,
    pub tout_ctrl: u8,
    pub clk_ctrl: u16,
    pub pwr_ctrl: u8,
    pub sw_rst: u8,
    pub normal_int: u16,
    pub error_int: u16,
    pub normal_int_stat_en: u16,
    pub error_int_stat_en: u16,
    pub normal_int_signal_en: u16,
    pub error_int_signal_en: u16,
    pub pstate: u32,
    pub cmd: u16,
    pub argument: u32,
    pub response01: u32,
    pub response23: u32,
    pub response45: u32,
    pub response67: u32,
    pub last_error: Option<SdCommandErrorReport>,
}

#[derive(Clone, Copy)]
pub struct SdRead0Report {
    pub status: &'static [u8],
    pub init_status: &'static [u8],
    pub rca: u16,
    pub ocr: u32,
    pub acmd41_attempts: u16,
    pub command_response: u32,
    pub last_error: Option<SdCommandErrorReport>,
    pub first_words: [u32; 8],
    pub mbr_signature: u16,
    pub partition_type: u8,
    pub normal_int: u16,
    pub error_int: u16,
    pub pstate: u32,
    pub block_size: u16,
    pub block_count: u16,
    pub xfer_mode: u16,
    pub cmd: u16,
    pub argument: u32,
}

type LispResult<T> = Result<T, Error>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i32),
    Word(u32),
    Symbol(SymbolId),
    Object(ObjectId),
    Primitive(Primitive),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Help,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    NumberEquals,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    Not,
    EqPredicate,
    NilPredicate,
    AtomPredicate,
    PairPredicate,
    NumberPredicate,
    SymbolPredicate,
    BoolPredicate,
    Cons,
    Car,
    Cdr,
    List,
    Led,
    Heartbeat,
    Button,
    Millis,
    Reg32,
    Poke32,
    Regs,
    SdStatus,
    SdPins,
    SdPinmux,
    SdClock,
    SdInit,
    SdRead0,
    SdhcRegs,
    Heap,
    Gc,
    Reboot,
}

impl Primitive {
    fn name(self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "mod",
            Self::NumberEquals => "=",
            Self::LessThan => "<",
            Self::LessOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterOrEqual => ">=",
            Self::Not => "not",
            Self::EqPredicate => "eq?",
            Self::NilPredicate => "nil?",
            Self::AtomPredicate => "atom?",
            Self::PairPredicate => "pair?",
            Self::NumberPredicate => "number?",
            Self::SymbolPredicate => "symbol?",
            Self::BoolPredicate => "bool?",
            Self::Cons => "cons",
            Self::Car => "car",
            Self::Cdr => "cdr",
            Self::List => "list",
            Self::Led => "led",
            Self::Heartbeat => "heartbeat",
            Self::Button => "button",
            Self::Millis => "millis",
            Self::Reg32 => "reg32",
            Self::Poke32 => "poke32",
            Self::Regs => "regs",
            Self::SdStatus => "sd-status",
            Self::SdPins => "sd-pins",
            Self::SdPinmux => "sd-pinmux",
            Self::SdClock => "sd-clock",
            Self::SdInit => "sd-init",
            Self::SdRead0 => "sd-read0",
            Self::SdhcRegs => "sdhc-regs",
            Self::Heap => "heap",
            Self::Gc => "gc",
            Self::Reboot => "reboot",
        }
    }
}

#[derive(Clone, Copy)]
enum ObjectKind {
    Free,
    Pair {
        car: Value,
        cdr: Value,
    },
    Closure {
        params: Value,
        body: Value,
        env: Value,
    },
    Env {
        symbol: SymbolId,
        value: Value,
        next: Value,
    },
}

#[derive(Clone, Copy)]
struct Object {
    marked: bool,
    next_free: Option<ObjectId>,
    kind: ObjectKind,
}

const FREE_OBJECT: Object = Object {
    marked: false,
    next_free: None,
    kind: ObjectKind::Free,
};

#[derive(Clone, Copy)]
struct SymbolEntry {
    occupied: bool,
    len: u8,
    bytes: [u8; MAX_SYMBOL_BYTES],
}

const EMPTY_SYMBOL: SymbolEntry = SymbolEntry {
    occupied: false,
    len: 0,
    bytes: [0; MAX_SYMBOL_BYTES],
};

#[derive(Clone, Copy)]
struct GlobalBinding {
    occupied: bool,
    symbol: SymbolId,
    value: Value,
}

const EMPTY_BINDING: GlobalBinding = GlobalBinding {
    occupied: false,
    symbol: 0,
    value: Value::Nil,
};

#[derive(Clone, Copy)]
struct SpecialSymbols {
    quote: SymbolId,
    if_: SymbolId,
    define: SymbolId,
    lambda: SymbolId,
    begin: SymbolId,
    let_: SymbolId,
    on: SymbolId,
    off: SymbolId,
    toggle: SymbolId,
    status: SymbolId,
}

const EMPTY_SPECIALS: SpecialSymbols = SpecialSymbols {
    quote: 0,
    if_: 0,
    define: 0,
    lambda: 0,
    begin: 0,
    let_: 0,
    on: 0,
    off: 0,
    toggle: 0,
    status: 0,
};

pub struct Machine {
    initialized: bool,
    symbols: [SymbolEntry; MAX_SYMBOLS],
    symbol_count: usize,
    globals: [GlobalBinding; MAX_GLOBALS],
    global_count: usize,
    objects: [Object; MAX_OBJECTS],
    free_head: Option<ObjectId>,
    specials: SpecialSymbols,
    active_expression: Value,
    collections: u32,
}

#[derive(Clone, Copy)]
struct HeapCounts {
    used: usize,
    free: usize,
    total: usize,
}

impl Machine {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            symbols: [EMPTY_SYMBOL; MAX_SYMBOLS],
            symbol_count: 0,
            globals: [EMPTY_BINDING; MAX_GLOBALS],
            global_count: 0,
            objects: [FREE_OBJECT; MAX_OBJECTS],
            free_head: None,
            specials: EMPTY_SPECIALS,
            active_expression: Value::Nil,
            collections: 0,
        }
    }

    pub fn bootstrap(&mut self) -> LispResult<()> {
        if self.initialized {
            return Ok(());
        }

        self.reset_heap();

        self.specials.quote = self.intern(b"quote")?;
        self.specials.if_ = self.intern(b"if")?;
        self.specials.define = self.intern(b"define")?;
        self.specials.lambda = self.intern(b"lambda")?;
        self.specials.begin = self.intern(b"begin")?;
        self.specials.let_ = self.intern(b"let")?;
        self.specials.on = self.intern(b"on")?;
        self.specials.off = self.intern(b"off")?;
        self.specials.toggle = self.intern(b"toggle")?;
        self.specials.status = self.intern(b"status")?;

        self.bind_self_evaluating_symbol(self.specials.on)?;
        self.bind_self_evaluating_symbol(self.specials.off)?;
        self.bind_self_evaluating_symbol(self.specials.toggle)?;
        self.bind_self_evaluating_symbol(self.specials.status)?;

        let true_symbol = self.intern(b"true")?;
        self.bind_global(true_symbol, Value::Bool(true))?;
        let false_symbol = self.intern(b"false")?;
        self.bind_global(false_symbol, Value::Bool(false))?;

        self.install_primitive(b"help", Primitive::Help)?;
        self.install_primitive(b"+", Primitive::Add)?;
        self.install_primitive(b"-", Primitive::Subtract)?;
        self.install_primitive(b"*", Primitive::Multiply)?;
        self.install_primitive(b"/", Primitive::Divide)?;
        self.install_primitive(b"mod", Primitive::Modulo)?;
        self.install_primitive(b"=", Primitive::NumberEquals)?;
        self.install_primitive(b"<", Primitive::LessThan)?;
        self.install_primitive(b"<=", Primitive::LessOrEqual)?;
        self.install_primitive(b">", Primitive::GreaterThan)?;
        self.install_primitive(b">=", Primitive::GreaterOrEqual)?;
        self.install_primitive(b"not", Primitive::Not)?;
        self.install_primitive(b"eq?", Primitive::EqPredicate)?;
        self.install_primitive(b"nil?", Primitive::NilPredicate)?;
        self.install_primitive(b"atom?", Primitive::AtomPredicate)?;
        self.install_primitive(b"pair?", Primitive::PairPredicate)?;
        self.install_primitive(b"number?", Primitive::NumberPredicate)?;
        self.install_primitive(b"symbol?", Primitive::SymbolPredicate)?;
        self.install_primitive(b"bool?", Primitive::BoolPredicate)?;
        self.install_primitive(b"cons", Primitive::Cons)?;
        self.install_primitive(b"car", Primitive::Car)?;
        self.install_primitive(b"cdr", Primitive::Cdr)?;
        self.install_primitive(b"list", Primitive::List)?;
        self.install_primitive(b"led", Primitive::Led)?;
        self.install_primitive(b"heartbeat", Primitive::Heartbeat)?;
        self.install_primitive(b"button", Primitive::Button)?;
        self.install_primitive(b"millis", Primitive::Millis)?;
        self.install_primitive(b"reg32", Primitive::Reg32)?;
        self.install_primitive(b"poke32", Primitive::Poke32)?;
        self.install_primitive(b"regs", Primitive::Regs)?;
        self.install_primitive(b"sd-status", Primitive::SdStatus)?;
        self.install_primitive(b"sd-pins", Primitive::SdPins)?;
        self.install_primitive(b"sd-pinmux", Primitive::SdPinmux)?;
        self.install_primitive(b"sd-clock", Primitive::SdClock)?;
        self.install_primitive(b"sd-init", Primitive::SdInit)?;
        self.install_primitive(b"sd-read0", Primitive::SdRead0)?;
        self.install_primitive(b"sdhc-regs", Primitive::SdhcRegs)?;
        self.install_primitive(b"heap", Primitive::Heap)?;
        self.install_primitive(b"gc", Primitive::Gc)?;
        self.install_primitive(b"reboot", Primitive::Reboot)?;

        self.initialized = true;
        Ok(())
    }

    pub fn eval_line<B: Board, W: Write>(
        &mut self,
        input: &[u8],
        board: &mut B,
        output: &mut W,
    ) -> fmt::Result {
        self.collect_garbage();

        let expression = match self.read(input) {
            Ok(expression) => expression,
            Err(error) => {
                writeln!(output, "error: {}", error.message())?;
                return Ok(());
            }
        };

        self.active_expression = expression;
        let result = self.eval(expression, Value::Nil, board, 0);
        self.active_expression = Value::Nil;

        match result {
            Ok(value) => {
                write!(output, "=> ")?;
                self.write_value(value, output)?;
                writeln!(output)?;
            }
            Err(error) => {
                writeln!(output, "error: {}", error.message())?;
            }
        }

        self.collect_garbage();
        Ok(())
    }

    fn reset_heap(&mut self) {
        let mut index = 0usize;
        while index < MAX_OBJECTS {
            let next = if index + 1 < MAX_OBJECTS {
                Some((index + 1) as ObjectId)
            } else {
                None
            };
            self.objects[index] = Object {
                marked: false,
                next_free: next,
                kind: ObjectKind::Free,
            };
            index += 1;
        }
        self.free_head = Some(0);
        self.collections = 0;
    }

    fn install_primitive(&mut self, name: &[u8], primitive: Primitive) -> LispResult<()> {
        let symbol = self.intern(name)?;
        self.bind_global(symbol, Value::Primitive(primitive))
    }

    fn bind_self_evaluating_symbol(&mut self, symbol: SymbolId) -> LispResult<()> {
        self.bind_global(symbol, Value::Symbol(symbol))
    }

    fn read(&mut self, input: &[u8]) -> LispResult<Value> {
        let mut reader = Reader { input, position: 0 };
        let expression = reader.read_expression(self)?;
        reader.skip_ws();
        if reader.is_done() {
            Ok(expression)
        } else {
            Err(Error::new("trailing input"))
        }
    }

    fn intern(&mut self, name: &[u8]) -> LispResult<SymbolId> {
        if name.is_empty() {
            return Err(Error::new("empty symbol"));
        }
        if name.len() > MAX_SYMBOL_BYTES {
            return Err(Error::new("symbol too long"));
        }

        let mut index = 0usize;
        while index < MAX_SYMBOLS {
            let entry = self.symbols[index];
            if entry.occupied && entry.len as usize == name.len() {
                let mut equal = true;
                let mut byte_index = 0usize;
                while byte_index < name.len() {
                    if entry.bytes[byte_index] != name[byte_index] {
                        equal = false;
                        break;
                    }
                    byte_index += 1;
                }

                if equal {
                    return Ok(index as SymbolId);
                }
            }
            index += 1;
        }

        if self.symbol_count >= MAX_SYMBOLS {
            return Err(Error::new("symbol table full"));
        }

        let id = self.symbol_count;
        let mut bytes = [0u8; MAX_SYMBOL_BYTES];
        let mut byte_index = 0usize;
        while byte_index < name.len() {
            bytes[byte_index] = name[byte_index];
            byte_index += 1;
        }

        self.symbols[id] = SymbolEntry {
            occupied: true,
            len: name.len() as u8,
            bytes,
        };
        self.symbol_count += 1;
        Ok(id as SymbolId)
    }

    fn bind_global(&mut self, symbol: SymbolId, value: Value) -> LispResult<()> {
        let mut index = 0usize;
        while index < MAX_GLOBALS {
            if self.globals[index].occupied && self.globals[index].symbol == symbol {
                self.globals[index].value = value;
                return Ok(());
            }
            index += 1;
        }

        index = 0;
        while index < MAX_GLOBALS {
            if !self.globals[index].occupied {
                self.globals[index] = GlobalBinding {
                    occupied: true,
                    symbol,
                    value,
                };
                self.global_count += 1;
                return Ok(());
            }
            index += 1;
        }

        Err(Error::new("global environment full"))
    }

    fn lookup(&self, symbol: SymbolId, env: Value) -> Option<Value> {
        let mut cursor = env;
        while let Value::Object(id) = cursor {
            let kind = self.object_kind_by_id(id).ok()?;
            match kind {
                ObjectKind::Env {
                    symbol: entry_symbol,
                    value,
                    next,
                } => {
                    if entry_symbol == symbol {
                        return Some(value);
                    }
                    cursor = next;
                }
                _ => return None,
            }
        }

        let mut index = 0usize;
        while index < MAX_GLOBALS {
            let binding = self.globals[index];
            if binding.occupied && binding.symbol == symbol {
                return Some(binding.value);
            }
            index += 1;
        }

        None
    }

    fn eval<B: Board>(
        &mut self,
        expression: Value,
        env: Value,
        board: &mut B,
        depth: u8,
    ) -> LispResult<Value> {
        if depth > MAX_EVAL_DEPTH {
            return Err(Error::new("evaluation depth limit"));
        }

        match expression {
            Value::Nil | Value::Bool(_) | Value::Int(_) | Value::Word(_) | Value::Primitive(_) => {
                Ok(expression)
            }
            Value::Symbol(symbol) => self.lookup(symbol, env).ok_or(Error::new("unbound symbol")),
            Value::Object(id) => match self.object_kind_by_id(id)? {
                ObjectKind::Pair { .. } => self.eval_call(expression, env, board, depth + 1),
                ObjectKind::Closure { .. } => Ok(expression),
                ObjectKind::Env { .. } => Err(Error::new("environment object is not a value")),
                ObjectKind::Free => Err(Error::new("stale object")),
            },
        }
    }

    fn eval_call<B: Board>(
        &mut self,
        expression: Value,
        env: Value,
        board: &mut B,
        depth: u8,
    ) -> LispResult<Value> {
        let operator = self.car(expression)?;
        let args = self.cdr(expression)?;

        if let Value::Symbol(symbol) = operator {
            if symbol == self.specials.quote {
                return self.form_quote(args);
            }
            if symbol == self.specials.if_ {
                return self.form_if(args, env, board, depth + 1);
            }
            if symbol == self.specials.define {
                return self.form_define(args, env, board, depth + 1);
            }
            if symbol == self.specials.lambda {
                return self.form_lambda(args, env);
            }
            if symbol == self.specials.begin {
                return self.eval_sequence(args, env, board, depth + 1);
            }
            if symbol == self.specials.let_ {
                return self.form_let(args, env, board, depth + 1);
            }
        }

        let function = self.eval(operator, env, board, depth + 1)?;
        self.apply(function, args, env, board, depth + 1)
    }

    fn form_quote(&self, args: Value) -> LispResult<Value> {
        let (value, rest) = self.require_pair(args)?;
        if rest != Value::Nil {
            return Err(Error::new("quote expects one argument"));
        }
        Ok(value)
    }

    fn form_if<B: Board>(
        &mut self,
        args: Value,
        env: Value,
        board: &mut B,
        depth: u8,
    ) -> LispResult<Value> {
        let (test, rest) = self.require_pair(args)?;
        let (consequent, rest) = self.require_pair(rest)?;
        let alternate = if rest == Value::Nil {
            Value::Nil
        } else {
            let (alternate, rest) = self.require_pair(rest)?;
            if rest != Value::Nil {
                return Err(Error::new("if expects two or three arguments"));
            }
            alternate
        };

        let test_value = self.eval(test, env, board, depth + 1)?;
        if self.truthy(test_value) {
            self.eval(consequent, env, board, depth + 1)
        } else {
            self.eval(alternate, env, board, depth + 1)
        }
    }

    fn form_define<B: Board>(
        &mut self,
        args: Value,
        env: Value,
        board: &mut B,
        depth: u8,
    ) -> LispResult<Value> {
        let (target, rest) = self.require_pair(args)?;

        if let Value::Symbol(symbol) = target {
            let (expression, rest) = self.require_pair(rest)?;
            if rest != Value::Nil {
                return Err(Error::new("define expects a name and one expression"));
            }

            let value = self.eval(expression, env, board, depth + 1)?;
            self.bind_global(symbol, value)?;
            return Ok(Value::Symbol(symbol));
        }

        if self.is_pair(target) {
            let name = self.car(target)?;
            let params = self.cdr(target)?;
            let symbol = match name {
                Value::Symbol(symbol) => symbol,
                _ => return Err(Error::new("function define needs a symbol name")),
            };

            if rest == Value::Nil {
                return Err(Error::new("function define needs a body"));
            }
            self.validate_params(params)?;
            let closure = self.alloc_object(ObjectKind::Closure {
                params,
                body: rest,
                env,
            })?;
            self.bind_global(symbol, closure)?;
            return Ok(Value::Symbol(symbol));
        }

        Err(Error::new(
            "define target must be a symbol or function form",
        ))
    }

    fn form_lambda(&mut self, args: Value, env: Value) -> LispResult<Value> {
        let (params, body) = self.require_pair(args)?;
        if body == Value::Nil {
            return Err(Error::new("lambda needs a body"));
        }
        self.validate_params(params)?;
        self.alloc_object(ObjectKind::Closure { params, body, env })
    }

    fn form_let<B: Board>(
        &mut self,
        args: Value,
        env: Value,
        board: &mut B,
        depth: u8,
    ) -> LispResult<Value> {
        let (bindings, body) = self.require_pair(args)?;
        if body == Value::Nil {
            return Err(Error::new("let needs a body"));
        }

        let mut new_env = env;
        let mut cursor = bindings;
        while let Some((binding, rest)) = self.list_next(cursor)? {
            let (name, value_tail) = self.require_pair(binding)?;
            let (value_expr, value_rest) = self.require_pair(value_tail)?;
            if value_rest != Value::Nil {
                return Err(Error::new("let binding expects a name and one value"));
            }
            let symbol = match name {
                Value::Symbol(symbol) => symbol,
                _ => return Err(Error::new("let binding name must be a symbol")),
            };
            let value = self.eval(value_expr, env, board, depth + 1)?;
            new_env = self.push_env(symbol, value, new_env)?;
            cursor = rest;
        }

        self.eval_sequence(body, new_env, board, depth + 1)
    }

    fn eval_sequence<B: Board>(
        &mut self,
        body: Value,
        env: Value,
        board: &mut B,
        depth: u8,
    ) -> LispResult<Value> {
        let mut result = Value::Nil;
        let mut cursor = body;
        while let Some((expression, rest)) = self.list_next(cursor)? {
            result = self.eval(expression, env, board, depth + 1)?;
            cursor = rest;
        }
        Ok(result)
    }

    fn apply<B: Board>(
        &mut self,
        function: Value,
        arg_expressions: Value,
        caller_env: Value,
        board: &mut B,
        depth: u8,
    ) -> LispResult<Value> {
        let mut args = [Value::Nil; MAX_CALL_ARGS];
        let arg_count =
            self.eval_arguments(arg_expressions, caller_env, board, depth + 1, &mut args)?;

        match function {
            Value::Primitive(primitive) => {
                self.apply_primitive(primitive, &args[..arg_count], caller_env, board)
            }
            Value::Object(id) => match self.object_kind_by_id(id)? {
                ObjectKind::Closure { params, body, env } => {
                    self.apply_closure(params, body, env, &args[..arg_count], board, depth + 1)
                }
                _ => Err(Error::new("value is not callable")),
            },
            _ => Err(Error::new("value is not callable")),
        }
    }

    fn eval_arguments<B: Board>(
        &mut self,
        expressions: Value,
        env: Value,
        board: &mut B,
        depth: u8,
        args: &mut [Value; MAX_CALL_ARGS],
    ) -> LispResult<usize> {
        let mut count = 0usize;
        let mut cursor = expressions;

        while let Some((expression, rest)) = self.list_next(cursor)? {
            if count >= MAX_CALL_ARGS {
                return Err(Error::new("too many call arguments"));
            }
            args[count] = self.eval(expression, env, board, depth + 1)?;
            count += 1;
            cursor = rest;
        }

        Ok(count)
    }

    fn apply_closure<B: Board>(
        &mut self,
        params: Value,
        body: Value,
        closure_env: Value,
        args: &[Value],
        board: &mut B,
        depth: u8,
    ) -> LispResult<Value> {
        let mut param_cursor = params;
        let mut arg_index = 0usize;
        let mut call_env = closure_env;

        while let Some((param, rest)) = self.list_next(param_cursor)? {
            if arg_index >= args.len() {
                return Err(Error::new("not enough arguments"));
            }
            let symbol = match param {
                Value::Symbol(symbol) => symbol,
                _ => return Err(Error::new("lambda parameter must be a symbol")),
            };
            call_env = self.push_env(symbol, args[arg_index], call_env)?;
            arg_index += 1;
            param_cursor = rest;
        }

        if arg_index != args.len() {
            return Err(Error::new("too many arguments"));
        }

        self.eval_sequence(body, call_env, board, depth + 1)
    }

    fn apply_primitive<B: Board>(
        &mut self,
        primitive: Primitive,
        args: &[Value],
        env: Value,
        board: &mut B,
    ) -> LispResult<Value> {
        match primitive {
            Primitive::Help => {
                self.expect_count(args, 0)?;
                self.help()
            }
            Primitive::Add => self.primitive_add(args),
            Primitive::Subtract => self.primitive_subtract(args),
            Primitive::Multiply => self.primitive_multiply(args),
            Primitive::Divide => self.primitive_divide(args),
            Primitive::Modulo => {
                self.expect_count(args, 2)?;
                let left = self.expect_int(args[0])?;
                let right = self.expect_int(args[1])?;
                if right == 0 {
                    return Err(Error::new("division by zero"));
                }
                Ok(Value::Int(left % right))
            }
            Primitive::NumberEquals => self.compare_numbers(args, |left, right| left == right),
            Primitive::LessThan => self.compare_numbers(args, |left, right| left < right),
            Primitive::LessOrEqual => self.compare_numbers(args, |left, right| left <= right),
            Primitive::GreaterThan => self.compare_numbers(args, |left, right| left > right),
            Primitive::GreaterOrEqual => self.compare_numbers(args, |left, right| left >= right),
            Primitive::Not => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(!self.truthy(args[0])))
            }
            Primitive::EqPredicate => {
                self.expect_count(args, 2)?;
                Ok(Value::Bool(args[0] == args[1]))
            }
            Primitive::NilPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(args[0] == Value::Nil))
            }
            Primitive::AtomPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(!self.is_pair(args[0])))
            }
            Primitive::PairPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(self.is_pair(args[0])))
            }
            Primitive::NumberPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(matches!(
                    args[0],
                    Value::Int(_) | Value::Word(_)
                )))
            }
            Primitive::SymbolPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
            }
            Primitive::BoolPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
            }
            Primitive::Cons => {
                self.expect_count(args, 2)?;
                self.alloc_pair(args[0], args[1])
            }
            Primitive::Car => {
                self.expect_count(args, 1)?;
                self.car(args[0])
            }
            Primitive::Cdr => {
                self.expect_count(args, 1)?;
                self.cdr(args[0])
            }
            Primitive::List => self.make_list_from_values(args),
            Primitive::Led => {
                self.expect_count(args, 1)?;
                let action = self.led_action(args[0])?;
                Ok(Value::Bool(board.led(action)))
            }
            Primitive::Heartbeat => {
                self.expect_count(args, 1)?;
                let enabled = self.on_off(args[0])?;
                Ok(Value::Bool(board.heartbeat(enabled)))
            }
            Primitive::Button => {
                self.expect_count(args, 1)?;
                let index = self.expect_int(args[0])?;
                Ok(Value::Bool(board.button_pressed(index)?))
            }
            Primitive::Millis => {
                self.expect_count(args, 0)?;
                Ok(Value::Int(board.millis() as i32))
            }
            Primitive::Reg32 => {
                self.expect_count(args, 1)?;
                let address = self.expect_word_address(args[0])?;
                let value = board.read32(address)?;
                Ok(Value::Word(value))
            }
            Primitive::Poke32 => {
                self.expect_count(args, 2)?;
                let address = self.expect_word_address(args[0])?;
                let value = self.expect_u32(args[1])?;
                board.write32(address, value)?;
                Ok(Value::Word(value))
            }
            Primitive::Regs => {
                self.expect_count(args, 0)?;
                self.register_report(board.registers())
            }
            Primitive::SdStatus => {
                self.expect_count(args, 0)?;
                self.sd_status_report(board.sd_status())
            }
            Primitive::SdPins => {
                self.expect_count(args, 0)?;
                self.sd_pins_report(board.sd_pins())
            }
            Primitive::SdPinmux => {
                self.expect_count(args, 0)?;
                self.sd_pins_report(board.sd_pinmux())
            }
            Primitive::SdClock => {
                self.expect_count(args, 0)?;
                self.sd_clock_report(board.sd_clock())
            }
            Primitive::SdInit => {
                self.expect_count(args, 0)?;
                self.sd_init_report(board.sd_init())
            }
            Primitive::SdRead0 => {
                self.expect_count(args, 0)?;
                self.sd_read0_report(board.sd_read0())
            }
            Primitive::SdhcRegs => {
                self.expect_count(args, 0)?;
                self.sdhc_report(board.sdhc_registers())
            }
            Primitive::Heap => {
                self.expect_count(args, 0)?;
                let counts = self.heap_counts();
                let values = [
                    Value::Int(counts.used as i32),
                    Value::Int(counts.free as i32),
                    Value::Int(counts.total as i32),
                    Value::Int(self.collections as i32),
                ];
                self.make_list_from_values(&values)
            }
            Primitive::Gc => {
                self.expect_count(args, 0)?;
                let freed = self.collect_garbage_from(env);
                Ok(Value::Int(freed as i32))
            }
            Primitive::Reboot => {
                self.expect_count(args, 0)?;
                board.reboot()
            }
        }
    }

    fn primitive_add(&self, args: &[Value]) -> LispResult<Value> {
        let mut result = 0i32;
        let mut index = 0usize;
        while index < args.len() {
            result = result
                .checked_add(self.expect_int(args[index])?)
                .ok_or(Error::new("integer overflow"))?;
            index += 1;
        }
        Ok(Value::Int(result))
    }

    fn primitive_subtract(&self, args: &[Value]) -> LispResult<Value> {
        if args.is_empty() {
            return Err(Error::new("- expects at least one argument"));
        }

        let mut result = self.expect_int(args[0])?;
        if args.len() == 1 {
            result = result.checked_neg().ok_or(Error::new("integer overflow"))?;
            return Ok(Value::Int(result));
        }

        let mut index = 1usize;
        while index < args.len() {
            result = result
                .checked_sub(self.expect_int(args[index])?)
                .ok_or(Error::new("integer overflow"))?;
            index += 1;
        }
        Ok(Value::Int(result))
    }

    fn primitive_multiply(&self, args: &[Value]) -> LispResult<Value> {
        let mut result = 1i32;
        let mut index = 0usize;
        while index < args.len() {
            result = result
                .checked_mul(self.expect_int(args[index])?)
                .ok_or(Error::new("integer overflow"))?;
            index += 1;
        }
        Ok(Value::Int(result))
    }

    fn primitive_divide(&self, args: &[Value]) -> LispResult<Value> {
        if args.len() < 2 {
            return Err(Error::new("/ expects at least two arguments"));
        }

        let mut result = self.expect_int(args[0])?;
        let mut index = 1usize;
        while index < args.len() {
            let divisor = self.expect_int(args[index])?;
            if divisor == 0 {
                return Err(Error::new("division by zero"));
            }
            result = result
                .checked_div(divisor)
                .ok_or(Error::new("integer overflow"))?;
            index += 1;
        }
        Ok(Value::Int(result))
    }

    fn compare_numbers<F>(&self, args: &[Value], compare: F) -> LispResult<Value>
    where
        F: Fn(i32, i32) -> bool,
    {
        if args.len() < 2 {
            return Err(Error::new("comparison expects at least two arguments"));
        }

        let mut previous = self.expect_int(args[0])?;
        let mut index = 1usize;
        while index < args.len() {
            let current = self.expect_int(args[index])?;
            if !compare(previous, current) {
                return Ok(Value::Bool(false));
            }
            previous = current;
            index += 1;
        }
        Ok(Value::Bool(true))
    }

    fn expect_count(&self, args: &[Value], expected: usize) -> LispResult<()> {
        if args.len() == expected {
            Ok(())
        } else {
            Err(Error::new("wrong argument count"))
        }
    }

    fn expect_int(&self, value: Value) -> LispResult<i32> {
        match value {
            Value::Int(value) => Ok(value),
            _ => Err(Error::new("expected integer")),
        }
    }

    fn expect_u32(&self, value: Value) -> LispResult<u32> {
        match value {
            Value::Word(value) => Ok(value),
            Value::Int(value) if value >= 0 => Ok(value as u32),
            _ => Err(Error::new("expected non-negative integer or word")),
        }
    }

    fn expect_word_address(&self, value: Value) -> LispResult<u32> {
        let address = self.expect_u32(value)?;
        if address & 0x03 != 0 {
            return Err(Error::new("register address must be word aligned"));
        }
        Ok(address)
    }

    fn led_action(&self, value: Value) -> LispResult<LedAction> {
        match value {
            Value::Symbol(symbol) if symbol == self.specials.on => Ok(LedAction::On),
            Value::Symbol(symbol) if symbol == self.specials.off => Ok(LedAction::Off),
            Value::Symbol(symbol) if symbol == self.specials.toggle => Ok(LedAction::Toggle),
            Value::Symbol(symbol) if symbol == self.specials.status => Ok(LedAction::Status),
            _ => Err(Error::new("led expects on, off, toggle, or status")),
        }
    }

    fn on_off(&self, value: Value) -> LispResult<bool> {
        match value {
            Value::Symbol(symbol) if symbol == self.specials.on => Ok(true),
            Value::Symbol(symbol) if symbol == self.specials.off => Ok(false),
            _ => Err(Error::new("expected on or off")),
        }
    }

    fn truthy(&self, value: Value) -> bool {
        !matches!(value, Value::Nil | Value::Bool(false))
    }

    fn validate_params(&self, params: Value) -> LispResult<()> {
        let mut cursor = params;
        while let Some((param, rest)) = self.list_next(cursor)? {
            if !matches!(param, Value::Symbol(_)) {
                return Err(Error::new("lambda parameters must be symbols"));
            }
            cursor = rest;
        }
        Ok(())
    }

    fn push_env(&mut self, symbol: SymbolId, value: Value, next: Value) -> LispResult<Value> {
        self.alloc_object(ObjectKind::Env {
            symbol,
            value,
            next,
        })
    }

    fn alloc_pair(&mut self, car: Value, cdr: Value) -> LispResult<Value> {
        self.alloc_object(ObjectKind::Pair { car, cdr })
    }

    fn alloc_object(&mut self, kind: ObjectKind) -> LispResult<Value> {
        let id = self.free_head.ok_or(Error::new("heap full"))?;
        let index = id as usize;
        self.free_head = self.objects[index].next_free;
        self.objects[index] = Object {
            marked: false,
            next_free: None,
            kind,
        };
        Ok(Value::Object(id))
    }

    fn make_list_from_values(&mut self, values: &[Value]) -> LispResult<Value> {
        let mut list = Value::Nil;
        let mut index = values.len();
        while index > 0 {
            index -= 1;
            list = self.alloc_pair(values[index], list)?;
        }
        Ok(list)
    }

    fn make_symbol_list(&mut self, names: &[&[u8]]) -> LispResult<Value> {
        let mut list = Value::Nil;
        let mut index = names.len();
        while index > 0 {
            index -= 1;
            let symbol = self.intern(names[index])?;
            list = self.alloc_pair(Value::Symbol(symbol), list)?;
        }
        Ok(list)
    }

    fn entry(&mut self, name: &[u8], value: Value) -> LispResult<Value> {
        let symbol = self.intern(name)?;
        self.alloc_pair(Value::Symbol(symbol), value)
    }

    fn symbol_entry(&mut self, name: &[u8], value: &[u8]) -> LispResult<Value> {
        let value = Value::Symbol(self.intern(value)?);
        self.entry(name, value)
    }

    fn bool_entry(&mut self, name: &[u8], value: bool) -> LispResult<Value> {
        self.entry(name, Value::Bool(value))
    }

    fn int_entry(&mut self, name: &[u8], value: i32) -> LispResult<Value> {
        self.entry(name, Value::Int(value))
    }

    fn word_entry(&mut self, name: &[u8], value: u32) -> LispResult<Value> {
        self.entry(name, Value::Word(value))
    }

    fn help(&mut self) -> LispResult<Value> {
        self.make_symbol_list(&[
            b"help",
            b"quote",
            b"if",
            b"define",
            b"lambda",
            b"begin",
            b"let",
            b"+",
            b"-",
            b"*",
            b"/",
            b"mod",
            b"=",
            b"<",
            b"<=",
            b">",
            b">=",
            b"not",
            b"eq?",
            b"nil?",
            b"atom?",
            b"pair?",
            b"number?",
            b"symbol?",
            b"bool?",
            b"cons",
            b"car",
            b"cdr",
            b"list",
            b"led",
            b"heartbeat",
            b"button",
            b"millis",
            b"reg32",
            b"poke32",
            b"regs",
            b"sd-status",
            b"sd-pins",
            b"sd-pinmux",
            b"sd-clock",
            b"sd-init",
            b"sd-read0",
            b"sdhc-regs",
            b"heap",
            b"gc",
            b"reboot",
        ])
    }

    fn register_report(&mut self, report: RegisterReport) -> LispResult<Value> {
        let scb5_ctrl = self.word_entry(b"SCB5.CTRL", report.scb5_ctrl)?;
        let scb5_uart_ctrl = self.word_entry(b"SCB5.UART_CTRL", report.scb5_uart_ctrl)?;
        let scb5_rx_status = self.word_entry(b"SCB5.RX_STATUS", report.scb5_rx_status)?;
        let scb5_tx_status = self.word_entry(b"SCB5.TX_STATUS", report.scb5_tx_status)?;
        let peri_clock5 = self.word_entry(b"PERI.CLOCK5", report.peri_clock5)?;
        let peri_div8_0 = self.word_entry(b"PERI.DIV8.0", report.peri_div8_0)?;
        let hsiom_prt5_sel0 = self.word_entry(b"HSIOM.PRT5.SEL0", report.hsiom_prt5_sel0)?;
        let gpio_prt5_cfg = self.word_entry(b"GPIO.PRT5.CFG", report.gpio_prt5_cfg)?;
        let gpio_prt13_out = self.word_entry(b"GPIO.PRT13.OUT", report.gpio_prt13_out)?;
        let gpio_prt13_cfg = self.word_entry(b"GPIO.PRT13.CFG", report.gpio_prt13_cfg)?;
        let entries = [
            scb5_ctrl,
            scb5_uart_ctrl,
            scb5_rx_status,
            scb5_tx_status,
            peri_clock5,
            peri_div8_0,
            hsiom_prt5_sel0,
            gpio_prt5_cfg,
            gpio_prt13_out,
            gpio_prt13_cfg,
        ];
        self.make_list_from_values(&entries)
    }

    fn sd_status_report(&mut self, report: SdStatusReport) -> LispResult<Value> {
        let cd_state: &[u8] = if report.cd_low { b"low" } else { b"high" };
        let cd_l = self.symbol_entry(b"CD_L", cd_state)?;
        let inserted = self.bool_entry(b"inserted", report.cd_low)?;
        let prt13_in = self.word_entry(b"GPIO.PRT13.IN", report.prt13_in)?;
        let prt13_cfg = self.word_entry(b"GPIO.PRT13.CFG", report.prt13_cfg)?;
        let entries = [cd_l, inserted, prt13_in, prt13_cfg];
        self.make_list_from_values(&entries)
    }

    fn sd_pins_report(&mut self, report: SdPinsReport) -> LispResult<Value> {
        let p12_sel1 = self.word_entry(b"P12.SEL1", report.p12_sel1)?;
        let p13_sel0 = self.word_entry(b"P13.SEL0", report.p13_sel0)?;
        let p12_cfg = self.word_entry(b"P12.CFG", report.p12_cfg)?;
        let p13_cfg = self.word_entry(b"P13.CFG", report.p13_cfg)?;
        let entries = [p12_sel1, p13_sel0, p12_cfg, p13_cfg];
        self.make_list_from_values(&entries)
    }

    fn sdhc_core_report(&mut self, report: SdhcCoreReport) -> LispResult<Value> {
        let wrap_ctl = self.word_entry(b"WRAP.CTL", report.wrap_ctl)?;
        let host_version = self.word_entry(b"HOST.VERSION", report.host_version as u32)?;
        let cap1 = self.word_entry(b"CAP1", report.cap1)?;
        let cap2 = self.word_entry(b"CAP2", report.cap2)?;
        let pstate = self.word_entry(b"PSTATE", report.pstate)?;
        let entries = [wrap_ctl, host_version, cap1, cap2, pstate];
        self.make_list_from_values(&entries)
    }

    fn sdhc_report(&mut self, report: SdhcReport) -> LispResult<Value> {
        let sdhc0 = self.sdhc_core_report(report.sdhc0)?;
        let sdhc1 = self.sdhc_core_report(report.sdhc1)?;
        let pins = self.sd_pins_report(report.pins)?;
        let sdhc0_section = self.entry(b"SDHC0", sdhc0)?;
        let sdhc1_section = self.entry(b"SDHC1", sdhc1)?;
        let pins_section = self.entry(b"microSD-pins", pins)?;
        let entries = [sdhc0_section, sdhc1_section, pins_section];
        self.make_list_from_values(&entries)
    }

    fn sd_clock_report(&mut self, report: SdClockReport) -> LispResult<Value> {
        let path0 = self.word_entry(b"CLK_PATH0", report.path0)?;
        let root0 = self.word_entry(b"CLK_HF0", report.root0)?;
        let root2 = self.word_entry(b"CLK_HF2", report.root2)?;
        let fll_config = self.word_entry(b"FLL_CONFIG", report.fll_config)?;
        let fll_config2 = self.word_entry(b"FLL_CONFIG2", report.fll_config2)?;
        let fll_status = self.word_entry(b"FLL_STATUS", report.fll_status)?;
        let selected_hf_hz = self.word_entry(b"selected-HF-Hz", report.selected_hf_hz)?;
        let entries = [
            path0,
            root0,
            root2,
            fll_config,
            fll_config2,
            fll_status,
            selected_hf_hz,
        ];
        self.make_list_from_values(&entries)
    }

    fn sd_init_report(&mut self, report: SdInitReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let cmd8 = self.word_entry(b"CMD8", report.cmd8_response)?;
        let cmd8_error = self.sd_error_code_entry(b"CMD8.error", report.cmd8_error)?;
        let acmd41_ocr = self.word_entry(b"ACMD41.OCR", report.acmd41_ocr)?;
        let attempts = self.int_entry(b"attempts", report.acmd41_attempts as i32)?;
        let clk_ctrl = self.word_entry(b"CLK_CTRL", report.clk_ctrl as u32)?;
        let normal_int = self.word_entry(b"NORM_INT", report.normal_int as u32)?;
        let error_int = self.word_entry(b"ERR_INT", report.error_int as u32)?;
        let pstate = self.word_entry(b"PSTATE", report.pstate)?;
        let cmd = self.word_entry(b"CMD_R", report.cmd as u32)?;
        let argument = self.word_entry(b"ARGUMENT", report.argument)?;
        let last_error = self.sd_error_code_entry(b"last-error", report.last_error)?;
        let pstate_error =
            self.sd_error_word_entry(b"PSTATE.error", report.last_error.map(|error| error.pstate))?;
        let pstate_after_write = self.sd_error_word_entry(
            b"PSTATE.after-write",
            report.last_error.map(|error| error.pstate_after_write),
        )?;
        let normal_int_after_write = self.sd_error_word_entry(
            b"NORM_INT.after-write",
            report
                .last_error
                .map(|error| error.normal_int_after_write as u32),
        )?;
        let error_int_after_write = self.sd_error_word_entry(
            b"ERR_INT.after-write",
            report
                .last_error
                .map(|error| error.error_int_after_write as u32),
        )?;
        let entries = [
            status,
            cmd8,
            cmd8_error,
            acmd41_ocr,
            attempts,
            clk_ctrl,
            normal_int,
            error_int,
            pstate,
            cmd,
            argument,
            last_error,
            pstate_error,
            pstate_after_write,
            normal_int_after_write,
            error_int_after_write,
        ];
        self.make_list_from_values(&entries)
    }

    fn sd_read0_report(&mut self, report: SdRead0Report) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let init_status = self.symbol_entry(b"init-status", report.init_status)?;
        let rca = self.word_entry(b"RCA", report.rca as u32)?;
        let ocr = self.word_entry(b"OCR", report.ocr)?;
        let attempts = self.int_entry(b"attempts", report.acmd41_attempts as i32)?;
        let response = self.word_entry(b"CMD17.response", report.command_response)?;
        let last_error = self.sd_error_code_entry(b"last-error", report.last_error)?;
        let first_words = self.sd_word_list_entry(b"first-words", &report.first_words)?;
        let mbr_signature = self.word_entry(b"MBR.sig", report.mbr_signature as u32)?;
        let partition_type = self.word_entry(b"partition0.type", report.partition_type as u32)?;
        let normal_int = self.word_entry(b"NORM_INT", report.normal_int as u32)?;
        let error_int = self.word_entry(b"ERR_INT", report.error_int as u32)?;
        let pstate = self.word_entry(b"PSTATE", report.pstate)?;
        let block_size = self.word_entry(b"BLOCK_SIZE", report.block_size as u32)?;
        let block_count = self.word_entry(b"BLOCK_COUNT", report.block_count as u32)?;
        let xfer_mode = self.word_entry(b"XFER_MODE", report.xfer_mode as u32)?;
        let cmd = self.word_entry(b"CMD_R", report.cmd as u32)?;
        let argument = self.word_entry(b"ARGUMENT", report.argument)?;
        let entries = [
            status,
            init_status,
            rca,
            ocr,
            attempts,
            response,
            last_error,
            first_words,
            mbr_signature,
            partition_type,
            normal_int,
            error_int,
            pstate,
            block_size,
            block_count,
            xfer_mode,
            cmd,
            argument,
        ];
        self.make_list_from_values(&entries)
    }

    fn sd_word_list_entry(&mut self, name: &[u8], words: &[u32]) -> LispResult<Value> {
        let mut values = [Value::Nil; 8];
        for (index, word) in words.iter().enumerate() {
            values[index] = Value::Word(*word);
        }
        let list = self.make_list_from_values(&values)?;
        self.entry(name, list)
    }

    fn sd_error_code_entry(
        &mut self,
        name: &[u8],
        report: Option<SdCommandErrorReport>,
    ) -> LispResult<Value> {
        match report {
            Some(error) => self.symbol_entry(name, error.code),
            None => self.entry(name, Value::Nil),
        }
    }

    fn sd_error_word_entry(&mut self, name: &[u8], value: Option<u32>) -> LispResult<Value> {
        match value {
            Some(value) => self.word_entry(name, value),
            None => self.entry(name, Value::Nil),
        }
    }

    fn is_pair(&self, value: Value) -> bool {
        match value {
            Value::Object(id) => matches!(self.object_kind_by_id(id), Ok(ObjectKind::Pair { .. })),
            _ => false,
        }
    }

    fn car(&self, value: Value) -> LispResult<Value> {
        match self.object_kind(value)? {
            ObjectKind::Pair { car, .. } => Ok(car),
            _ => Err(Error::new("expected pair")),
        }
    }

    fn cdr(&self, value: Value) -> LispResult<Value> {
        match self.object_kind(value)? {
            ObjectKind::Pair { cdr, .. } => Ok(cdr),
            _ => Err(Error::new("expected pair")),
        }
    }

    fn set_cdr(&mut self, pair: Value, cdr: Value) -> LispResult<()> {
        let id = match pair {
            Value::Object(id) => id,
            _ => return Err(Error::new("expected pair")),
        };
        let index = id as usize;
        let car = match self.object_kind_by_id(id)? {
            ObjectKind::Pair { car, .. } => car,
            _ => return Err(Error::new("expected pair")),
        };
        self.objects[index].kind = ObjectKind::Pair { car, cdr };
        Ok(())
    }

    fn object_kind(&self, value: Value) -> LispResult<ObjectKind> {
        match value {
            Value::Object(id) => self.object_kind_by_id(id),
            _ => Err(Error::new("expected heap object")),
        }
    }

    fn object_kind_by_id(&self, id: ObjectId) -> LispResult<ObjectKind> {
        let index = id as usize;
        if index >= MAX_OBJECTS {
            return Err(Error::new("invalid object"));
        }
        match self.objects[index].kind {
            ObjectKind::Free => Err(Error::new("stale object")),
            kind => Ok(kind),
        }
    }

    fn require_pair(&self, value: Value) -> LispResult<(Value, Value)> {
        match self.object_kind(value)? {
            ObjectKind::Pair { car, cdr } => Ok((car, cdr)),
            _ => Err(Error::new("expected proper list")),
        }
    }

    fn list_next(&self, cursor: Value) -> LispResult<Option<(Value, Value)>> {
        if cursor == Value::Nil {
            return Ok(None);
        }
        match self.object_kind(cursor)? {
            ObjectKind::Pair { car, cdr } => Ok(Some((car, cdr))),
            _ => Err(Error::new("expected proper list")),
        }
    }

    fn heap_counts(&self) -> HeapCounts {
        let mut free = 0usize;
        let mut index = 0usize;
        while index < MAX_OBJECTS {
            if matches!(self.objects[index].kind, ObjectKind::Free) {
                free += 1;
            }
            index += 1;
        }
        HeapCounts {
            used: MAX_OBJECTS - free,
            free,
            total: MAX_OBJECTS,
        }
    }

    fn collect_garbage(&mut self) -> usize {
        self.collect_garbage_from(Value::Nil)
    }

    fn collect_garbage_from(&mut self, env: Value) -> usize {
        let mut index = 0usize;
        while index < MAX_GLOBALS {
            let binding = self.globals[index];
            if binding.occupied {
                self.mark_value(binding.value);
            }
            index += 1;
        }

        self.mark_value(self.active_expression);
        self.mark_value(env);

        let mut freed = 0usize;
        index = 0;
        while index < MAX_OBJECTS {
            if matches!(self.objects[index].kind, ObjectKind::Free) {
                index += 1;
                continue;
            }

            if self.objects[index].marked {
                self.objects[index].marked = false;
            } else {
                self.objects[index] = Object {
                    marked: false,
                    next_free: self.free_head,
                    kind: ObjectKind::Free,
                };
                self.free_head = Some(index as ObjectId);
                freed += 1;
            }

            index += 1;
        }

        self.collections = self.collections.wrapping_add(1);
        freed
    }

    fn mark_value(&mut self, value: Value) {
        let id = match value {
            Value::Object(id) => id,
            _ => return,
        };

        let index = id as usize;
        if index >= MAX_OBJECTS || self.objects[index].marked {
            return;
        }

        self.objects[index].marked = true;
        match self.objects[index].kind {
            ObjectKind::Pair { car, cdr } => {
                self.mark_value(car);
                self.mark_value(cdr);
            }
            ObjectKind::Closure { params, body, env } => {
                self.mark_value(params);
                self.mark_value(body);
                self.mark_value(env);
            }
            ObjectKind::Env { value, next, .. } => {
                self.mark_value(value);
                self.mark_value(next);
            }
            ObjectKind::Free => {}
        }
    }

    fn write_value<W: Write>(&self, value: Value, output: &mut W) -> fmt::Result {
        match value {
            Value::Nil => output.write_str("nil"),
            Value::Bool(true) => output.write_str("#t"),
            Value::Bool(false) => output.write_str("#f"),
            Value::Int(value) => write!(output, "{}", value),
            Value::Word(value) => write!(output, "#x{:08x}", value),
            Value::Symbol(symbol) => self.write_symbol(symbol, output),
            Value::Primitive(primitive) => write!(output, "<primitive {}>", primitive.name()),
            Value::Object(id) => match self.object_kind_by_id(id) {
                Ok(ObjectKind::Pair { .. }) => self.write_pair(id, output),
                Ok(ObjectKind::Closure { .. }) => output.write_str("<lambda>"),
                Ok(ObjectKind::Env { .. }) => output.write_str("<env>"),
                _ => output.write_str("<stale>"),
            },
        }
    }

    fn write_pair<W: Write>(&self, id: ObjectId, output: &mut W) -> fmt::Result {
        output.write_char('(')?;
        let mut cursor = Value::Object(id);
        let mut first = true;

        loop {
            match self.object_kind(cursor) {
                Ok(ObjectKind::Pair { car, cdr }) => {
                    if !first {
                        output.write_char(' ')?;
                    }
                    self.write_value(car, output)?;
                    first = false;

                    match cdr {
                        Value::Nil => {
                            output.write_char(')')?;
                            return Ok(());
                        }
                        Value::Object(next_id)
                            if matches!(
                                self.object_kind_by_id(next_id),
                                Ok(ObjectKind::Pair { .. })
                            ) =>
                        {
                            cursor = cdr;
                        }
                        value => {
                            output.write_str(" . ")?;
                            self.write_value(value, output)?;
                            output.write_char(')')?;
                            return Ok(());
                        }
                    }
                }
                _ => {
                    output.write_str("<bad-list>)")?;
                    return Ok(());
                }
            }
        }
    }

    fn write_symbol<W: Write>(&self, symbol: SymbolId, output: &mut W) -> fmt::Result {
        let index = symbol as usize;
        if index >= MAX_SYMBOLS || !self.symbols[index].occupied {
            return output.write_str("<bad-symbol>");
        }

        let entry = self.symbols[index];
        let mut byte_index = 0usize;
        while byte_index < entry.len as usize {
            output.write_char(entry.bytes[byte_index] as char)?;
            byte_index += 1;
        }
        Ok(())
    }
}

struct Reader<'a> {
    input: &'a [u8],
    position: usize,
}

impl Reader<'_> {
    fn read_expression(&mut self, machine: &mut Machine) -> LispResult<Value> {
        self.skip_ws();
        match self.peek() {
            Some(b'(') => {
                self.position += 1;
                self.read_list(machine)
            }
            Some(b')') => Err(Error::new("unexpected ')'")),
            Some(b'\'') => {
                self.position += 1;
                let quoted = self.read_expression(machine)?;
                let quote_tail = machine.alloc_pair(quoted, Value::Nil)?;
                machine.alloc_pair(Value::Symbol(machine.specials.quote), quote_tail)
            }
            Some(_) => self.read_atom(machine),
            None => Err(Error::new("unexpected end of input")),
        }
    }

    fn read_list(&mut self, machine: &mut Machine) -> LispResult<Value> {
        let mut head = Value::Nil;
        let mut tail = Value::Nil;

        loop {
            self.skip_ws();
            match self.peek() {
                Some(b')') => {
                    self.position += 1;
                    return Ok(head);
                }
                Some(b'.') => {
                    self.position += 1;
                    if tail == Value::Nil {
                        return Err(Error::new("dot needs a list head"));
                    }
                    let cdr = self.read_expression(machine)?;
                    self.skip_ws();
                    self.expect(b')')?;
                    machine.set_cdr(tail, cdr)?;
                    return Ok(head);
                }
                Some(_) => {
                    let value = self.read_expression(machine)?;
                    let cell = machine.alloc_pair(value, Value::Nil)?;
                    if head == Value::Nil {
                        head = cell;
                    } else {
                        machine.set_cdr(tail, cell)?;
                    }
                    tail = cell;
                }
                None => return Err(Error::new("unterminated list")),
            }
        }
    }

    fn read_atom(&mut self, machine: &mut Machine) -> LispResult<Value> {
        let start = self.position;
        while let Some(byte) = self.peek() {
            if is_delimiter(byte) {
                break;
            }
            self.position += 1;
        }

        let token = &self.input[start..self.position];
        if token == b"nil" {
            return Ok(Value::Nil);
        }
        if token == b"#t" {
            return Ok(Value::Bool(true));
        }
        if token == b"#f" {
            return Ok(Value::Bool(false));
        }
        if let Some(value) = parse_decimal(token)? {
            return Ok(Value::Int(value));
        }
        if let Some(value) = parse_hex(token)? {
            return Ok(Value::Word(value));
        }

        Ok(Value::Symbol(machine.intern(token)?))
    }

    fn skip_ws(&mut self) {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
                self.position += 1;
            }

            if self.peek() == Some(b';') {
                while let Some(byte) = self.peek() {
                    self.position += 1;
                    if byte == b'\n' {
                        break;
                    }
                }
                continue;
            }

            break;
        }
    }

    fn expect(&mut self, byte: u8) -> LispResult<()> {
        if self.peek() == Some(byte) {
            self.position += 1;
            Ok(())
        } else {
            Err(Error::new("unexpected syntax"))
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    fn is_done(&self) -> bool {
        self.position >= self.input.len()
    }
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\t' | b'\r' | b'\n' | b'(' | b')' | b'\'' | b';'
    )
}

fn parse_decimal(token: &[u8]) -> LispResult<Option<i32>> {
    if token.is_empty() {
        return Ok(None);
    }

    let mut index = 0usize;
    let mut negative = false;
    if token[0] == b'-' {
        negative = true;
        index = 1;
        if index == token.len() {
            return Ok(None);
        }
    }

    let mut value = 0i64;
    while index < token.len() {
        let byte = token[index];
        if !byte.is_ascii_digit() {
            return Ok(None);
        }
        value = value * 10 + (byte - b'0') as i64;
        index += 1;
    }

    if negative {
        value = -value;
    }

    if value < i32::MIN as i64 || value > i32::MAX as i64 {
        return Err(Error::new("integer overflow"));
    }

    Ok(Some(value as i32))
}

fn parse_hex(token: &[u8]) -> LispResult<Option<u32>> {
    if token.len() < 3 || token[0] != b'#' || !matches!(token[1], b'x' | b'X') {
        return Ok(None);
    }

    let mut value = 0u32;
    let mut index = 2usize;
    while index < token.len() {
        let digit = hex_digit(token[index]).ok_or(Error::new("invalid hex integer"))?;
        value = value
            .checked_mul(16)
            .and_then(|value| value.checked_add(digit as u32))
            .ok_or(Error::new("integer overflow"))?;
        index += 1;
    }

    Ok(Some(value))
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + byte - b'a'),
        b'A'..=b'F' => Some(10 + byte - b'A'),
        _ => None,
    }
}
