use std::collections::HashMap;
use xcb_util::{ewmh, icccm, keysyms::KeySymbols};
use crate::keys::{self, Command, KeyCombo};
use std::time::{SystemTime, UNIX_EPOCH};


pub struct Client {
    pub connection: ewmh::Connection,

    pub root_window: xcb::Window,
    pub screen_idx: i32,

    pub keymap: HashMap<keys::KeyCombo, keys::Command>,
}

impl Client {
    pub fn open_connection() -> Self {
        let (connection, screen_idx) = xcb::Connection::connect(None).unwrap();
        let connection = ewmh::Connection::connect(connection)
            .map_err(|(e, _)| e).unwrap();

        let root_window = connection.get_setup()
            .roots().nth(screen_idx as usize)
            .ok_or("Invalid screen").unwrap().root();

        Self {
            connection,
            screen_idx,
            root_window,
            keymap: HashMap::new(),
        }
    }

    pub fn register_keybinds(
        &mut self,
        keybinds: Vec<(KeyCombo, Command)>
    ) {
        let symbols = KeySymbols::new(&self.connection);

        for (combo, command) in keybinds {
            self.keymap.insert(combo, command);
            println!("Combo key: {}", combo.key);

            if let Some(code) = symbols.get_keycode(combo.key).next() {
                xcb::grab_key(
                    &self.connection,
                    false,
                    self.root_window,
                    combo.mods as u16,
                    code,
                    // xcb::MOD_MASK_ANY as u16,
                    // xcb::GRAB_ANY as u8,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::GRAB_MODE_ASYNC as u8,
                );
            }
        }
    }

    pub fn poll(&mut self) -> Event {
        loop {
            self.connection.flush();
            let event = self.connection.wait_for_event()
                .expect("Wait for event returned none");

            match event.response_type() {
                xcb::KEY_PRESS => {
                    let event = unsafe { xcb::cast_event::<xcb::KeyPressEvent>(&event) };
                    let syms = KeySymbols::new(&self.connection);
                    let key = syms.press_lookup_keysym(event, 0);
                    let mods = u32::from(event.state());

                    println!("{:?} {:?} PRESS",
                             SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                             key);


                    let combo = KeyCombo { mods, key, event: keys::Event::KeyDown };
                    if let Some(command) = self.keymap.get(&combo) {
                        return Event::Command(command.clone());
                    }
                },
                xcb::KEY_RELEASE => {
                    let event = unsafe { xcb::cast_event::<xcb::KeyPressEvent>(&event) };
                    let syms = KeySymbols::new(&self.connection);
                    let key = syms.press_lookup_keysym(event, 0);
                    let mods = u32::from(event.state());

                    println!("{:?} {:?} RELEASE",
                             SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                             key);

                    let combo = KeyCombo { mods, key, event: keys::Event::KeyUp };
                    if let Some(command) = self.keymap.get(&combo) {
                        return Event::Command(command.clone());
                    }
                },
                _ => {},
            }
        }
    }
}

pub enum Event {
    Command(keys::Command)
}
