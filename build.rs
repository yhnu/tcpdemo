use std::io::Result;
fn main() -> Result<()> {
   let mut prost_build = prost_build::Config::new();
   prost_build.out_dir("src");
   prost_build.compile_protos(&["src/message.proto"], &["src"])?;
   Ok(())
}