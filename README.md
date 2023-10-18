# winit-gtk

`winit-gtk` is winit fork with GTK backend. While we are waiting [winit#2430](https://github.com/rust-windowing/winit/pull/2430), you can use this crate already by adding cargo patch to your project. 

The minor version of `winit-gtk` should match to the exact `winit` version. Here is the compatibility table:
| winit  | winit-gtk |
| :-:    | :-------: |
| 0.28.7 | 0.29      |
| 0.28.6 | 0.28      |


## Usage

GTK can be initialized in any thread, but the context must be in the same thread. `winit-gtk` makes sure `Window` and other proxy types can work in multiple thread. But if you want to call GTK methods yourself, it must be called in the same thread where event loop is created. Otherwise, GTK will panic.

`winit-gtk` will try to keep the same APIs as `winit`, but there are still some missing features. See tracking issues to know more info. The feature flags are also as same as `winit`, but `x11` and `wayland` platform modules are replaced with `gtk` module.

`winit-gtk` is implemented in the way that can work with `winit`'s control flow variants. It is indeed not the best way to work with GTK's main context IMHO. We are welcome anyone who is interested to help us improve, fix bugs, and fill out missing features.

