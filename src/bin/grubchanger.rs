use modularitea_libs::helper::grubchanger::GrubChangerContext;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    settheme: Option<String>
}

fn main() {
    let args = Args::parse();
    if let Some(args_val) = args.settheme {

        // place libs code here, for example.
        let ctx = GrubChangerContext::new(String::from("aaa"));
        ctx.apply();
        unimplemented!();

        // why this is separated program? because we cant rely only on libs (that included with)
        // tealinux modularitea itself, we need to exec polkit here, so for this privileged stuff
        // must be sparated //
    }
}