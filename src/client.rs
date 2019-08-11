use xcb;
use std::collections::HashMap;
use xcb_util::{ewmh, keysyms::KeySymbols};
use crate::keys::{self, Command, KeyCombo};
use std::time::{SystemTime, UNIX_EPOCH};


// TODO Rename / repurpose to be just X11 support

pub struct Client {
    pub connection: ewmh::Connection,

    pub root_window: xcb::Window,
    pub screen_idx: i32,

    pub keymap: HashMap<keys::KeyCombo, keys::Command>,

}



impl Client {
    pub fn open_connection() -> Self {
        println!("Opening connection...");
        let (connection, screen_idx) = xcb::Connection::connect(None).expect("Failed to start xcb connection");
        let connection = ewmh::Connection::connect(connection)
            .map_err(|(e, _)| e).expect("Failed to start ewmh connection");

        let root_window = connection.get_setup()
            .roots().nth(screen_idx as usize)
            .ok_or("Invalid screen").expect("Failed to get setup...").root();

        Self {
            connection,
            screen_idx,
            root_window,
            keymap: HashMap::new(),
        }
    }

    // xcb::GRAB_ANY as u8,
    pub fn register_keyboard(&mut self) {
        xcb::grab_keyboard(
            &self.connection,
            false,
            self.root_window,
            xcb::CURRENT_TIME,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        );
    }

    pub fn unregister_keyboard(&mut self) {
        xcb::ungrab_keyboard(
            &self.connection,
            xcb::CURRENT_TIME
        );
    }

    pub fn register_keymap(
        &mut self,
        keybinds: Vec<(KeyCombo, Command)>,
        full_grab: bool
    ) {
        if full_grab {
            self.register_keyboard();
        }
        else {
            self.unregister_keyboard();
        }

        let symbols = KeySymbols::new(&self.connection);
        for (combo, command) in keybinds {
            self.keymap.insert(combo, command);
            println!("Combo key: {}", combo.key);

            if !full_grab {
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

                    println!("{:?} {:?} {:?} PRESS",
                             SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                             key, mods);


                    let combo = KeyCombo { mods, key, event: keys::Event::KeyDown };
                    if let Some(command) = self.keymap.get(&combo) {
                        return Event::Command(command.clone());
                    }
                    // Key combo is not in the map or pressed modifier
                    //             Super_L         Super_R
                    else if key != 65515 && key != 65515
                    //             Ctrl_L          Ctrl_R
                         && key != 65507 && key != 65508
                    //             Caps            Function
                         && key != 65509 && key != 269025067
                    //             Shift_L         Shift_R
                         && key != 65505 && key != 65506
                    //             Alt_L           Alt_R
                         && key != 65513 && key != 65514
                    {
                        let command = Command::Noop();
                        return Event::Command(command);
                    }
                },
                xcb::KEY_RELEASE => {
                    let event = unsafe { xcb::cast_event::<xcb::KeyPressEvent>(&event) };
                    let syms = KeySymbols::new(&self.connection);
                    let key = syms.press_lookup_keysym(event, 0);
                    let mods = u32::from(event.state());

                    println!("{:?} {:?} {:?} RELEASE",
                             SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                             key, mods);

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
