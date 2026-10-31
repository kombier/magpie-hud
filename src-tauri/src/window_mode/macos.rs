//! Retain the native traffic lights, routing green through Tao's floating-window-safe fullscreen API.

use std::cell::RefCell;
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadOnly};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSWindow, NSWindowButton, NSWindowCollectionBehavior as Behavior};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use tauri::WebviewWindow;

define_class!(
    // SAFETY: NSObject imposes no subclassing requirements; this class has no custom Drop.
    #[unsafe(super = NSObject)]
    #[name = "MagpieHudFullscreenTarget"]
    #[thread_kind = MainThreadOnly]
    #[ivars = WebviewWindow]
    struct FullscreenTarget;

    unsafe impl NSObjectProtocol for FullscreenTarget {}

    impl FullscreenTarget {
        // SAFETY: matches the AppKit target/action signature, and runs on the main thread.
        #[unsafe(method(toggleHudFullscreen:))]
        fn toggle(&self, _sender: Option<&AnyObject>) {
            let window = self.ivars();
            let result = window.is_fullscreen().and_then(|fullscreen| window.set_fullscreen(!fullscreen));
            if let Err(error) = result { eprintln!("Native fullscreen failed: {error}"); }
        }
    }
);

thread_local! {
    // NSControl does not retain its target. There is exactly one main HUD window.
    static GREEN_TARGET: RefCell<Option<Retained<FullscreenTarget>>> = const { RefCell::new(None) };
}

pub(super) fn configure(window: WebviewWindow) {
    let mtm = MainThreadMarker::new().expect("AppKit window configuration must run on the main thread");
    let pointer = window.ns_window().expect("Main window must have an AppKit window");
    // SAFETY: Tauri owns this live NSWindow and this function runs on its main thread.
    let native = unsafe { &*pointer.cast::<NSWindow>() };
    let mut behavior = native.collectionBehavior();
    behavior.remove(Behavior::FullScreenAuxiliary | Behavior::FullScreenNone);
    behavior.insert(Behavior::FullScreenPrimary);
    native.setCollectionBehavior(behavior);
    let button = native.standardWindowButton(NSWindowButton::ZoomButton).expect("Missing native green button");
    let target = FullscreenTarget::alloc(mtm).set_ivars(window);
    // SAFETY: NSObject init has this signature and initializes our allocated subclass.
    let target: Retained<FullscreenTarget> = unsafe { msg_send![super(target), init] };
    // SAFETY: the selector is defined above with the required target/action signature.
    // Tao lowers the floating window before entering fullscreen; AppKit's default zoom action does not.
    unsafe {
        button.setTarget(Some(&target));
        button.setAction(Some(sel!(toggleHudFullscreen:)));
    }
    GREEN_TARGET.with(|slot| *slot.borrow_mut() = Some(target));
}
