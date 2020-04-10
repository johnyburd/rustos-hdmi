use core::panic::PanicInfo;
use crate::console::kprintln;

const PANIC_BANNER : &str = 
"
             (
       (      )     )
         )   (    (
        (          `
    .-\"\"^\"\"\"^\"\"^\"\"\"^\"\"-.
  (//\\\\//\\\\//\\\\//\\\\//\\\\//)
   ~\\^^^^^^^^^^^^^^^^^^/~
     `================`

    The pi is overdone.

---------- PANIC ----------

";

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("{}", PANIC_BANNER);
    if let Some(s) = _info.payload().downcast_ref::<&str>() {
        kprintln!("payload: {:?}", s);
    } else {
        kprintln!("payload: no payload");
    }

    if let Some(location) = _info.location() {
        kprintln!("panic occurred in file '{}' at line {}", location.file(),
            location.line());
    } else {
        kprintln!("can't get location information...");
    }

    loop { };
}
