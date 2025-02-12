// *** MathCAT doesn't normally want to build a binary ***
// *** This file is here because it is useful for trying out things ***


// const LOG_FILE: &str = "./MathCAT.log";
// lazy_static! {
//   static ref LOG: slog::Logger = create_log();
// }

// fn create_log() -> slog::Logger {
  // let file = OpenOptions::new()
  //   .create(true)
  //   .write(true)
  //   .truncate(true)
  //   .open(LOG_FILE)
  //   .unwrap();

  // let decorator = slog_term::PlainDecorator::new(file);
  // let drain = slog_term::FullFormat::new(decorator).build().fuse();
  // let drain = slog_async::Async::new(drain).build().fuse();

  // let logger = slog::Logger::root(drain, o!());
  // return logger;
// }

// Maybe also have this speak to test the TTS generation.
// There is a rust winapi crate that mirrors the WinPAI and has "Speak(...)" in it
fn main() {
  use libmathcat::interface::*;
  use log::*;
  use std::time::{Instant};
  env_logger::builder()
      .format_timestamp(None)
      .format_module_path(false)
      .format_indent(None)
      .format_level(false)
      .init();

  //  let expr = "
  //     <math display='block' xmlns='http://www.w3.org/1998/Math/MathML'>
  //     <mrow>
  //      <mrow><mo>[</mo>
  //        <mtable>
  //         <mtr>
  //          <mtd>
  //           <mn>3</mn>
  //          </mtd>
  //          <mtd>
  //           <mn>1</mn>
  //          </mtd>
  //          <mtd>
  //           <mn>4</mn>
  //          </mtd>
  //         </mtr>
  //         <mtr>
  //          <mtd>
  //           <mn>0</mn>
  //          </mtd>
  //          <mtd>
  //           <mn>2</mn>
  //          </mtd>
  //          <mtd>
  //           <mn>6</mn>
  //          </mtd>
  //         </mtr>
  //        </mtable>
  //      <mo>]</mo></mrow></mrow>
  //    </math>
  // ";

  let expr = "<math display='inline' xmlns='http://www.w3.org/1998/Math/MathML' subject='None'>
  <mrow>
  <mo>(</mo><mn arg='arg1'>3</mn><mo>,</mo><mn arg='arg2'>12</mn><mo>)</mo>
  </mrow>
  </math>
  
      ";
  // let expr = "
  // <math><mo>&#x25B3;</mo><mi>ABC</mi></math>
  //   ";
  // let expr = "<math><mi>c</mi><mo>=</mo><mn>4</mn><mspace width=\"thinmathspace\"></mspace><mn>598</mn>
  //                 <mspace width=\"thinmathspace\"></mspace><mn>037</mn>
  //                 <mspace width=\"thinmathspace\"></mspace><mn>234</mn></math>";
  let instant = Instant::now();
  let rules_dir = std::env::current_exe().unwrap().parent().unwrap().join("../../../Rules");
  let rules_dir = rules_dir.as_os_str().to_str().unwrap().to_string();
  if let Err(e) = set_rules_dir(rules_dir) {
    panic!("Error: exiting -- {}", errors_to_string(&e));  }
  if let Err(e) = set_mathml(expr.to_string()) {
    panic!("Error: exiting -- {}", errors_to_string(&e));
  };

  info!("Version = '{}'", get_version());
  set_preference("TTS".to_string(), "none".to_string()).unwrap();
  // set_preference("Bookmark".to_string(), "true".to_string()).unwrap();
  set_preference("SpeechStyle".to_string(), "ClearSpeak".to_string()).unwrap();

  match get_spoken_text() {
    Ok(speech) => info!("Computed speech string:\n   '{}'", speech),
    Err(e) => panic!("{}", errors_to_string(&e)),
  }
  info!("SpeechStyle: {:?}", get_preference("SpeechStyle".to_string()).unwrap());
 
  set_preference("BrailleCode".to_string(), "Nemeth".to_string()).unwrap();
  match get_braille("".to_string()) {
    Ok(braille) => info!("Computed braille string:\n   '{}'", braille),
    Err(e) => panic!("{}", errors_to_string(&e)),
  }

  // Note: the logger seems to be a huge time sync, so println! is used for timing
  info!("Time taken: {}ms", instant.elapsed().as_millis());
  // let instant = Instant::now();
  // set_preference("SpeechStyle".to_string(), "ClearSpeak".to_string()).unwrap();
  // match get_spoken_text() {
  //   Ok(speech) => info!("Computed speech string:\n   '{}'", speech),
  //   Err(e) => panic!("{}", errors_to_string(&e)),
  // }
  // info!("SpeechStyle: {:?}", get_preference("SpeechStyle".to_string()));
  
  // match get_braille("".to_string()) {
  //   Ok(braille) => info!("Computed braille string:\n   '{}'", braille),
  //   Err(e) => panic!("{}", errors_to_string(&e)),
  // }
  // info!("Time taken (second time): {}ms", instant.elapsed().as_millis());
}
