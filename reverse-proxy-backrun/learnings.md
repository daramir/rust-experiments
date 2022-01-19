- move
- match
- ARCs
- hyper
- module scoping (settings_mod on main to make available via `crate` to other modules)
  
```md
  If you don't want to put myutils directly inside the crate root (main.rs or lib.rs), because myutils contains stuff that's only relevant to some_ui and other_ui, then consider scoping those together under another module:
  src/
    ├── main.rs
    └── ui
        ├── other.rs
        ├── some.rs
        └── utils.rs
    Then in main.rs you might have

    pub mod ui {
        pub mod other;
        pub mod some;
        mod utils; // because it's not `pub` it won't be visible outside of `ui`
    }
    and in other.rs and some.rs you can use the utils module from their shared parent:

    use super::utils;
```
    