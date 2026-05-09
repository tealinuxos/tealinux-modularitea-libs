use ini::Ini;

fn main() {
    let path = "/home/fadhil_riyanto_guest/BALI64/tealinux-modularitea-libs2/testfiles/grub";

    let mut i = Ini::load_from_file(path).unwrap();
    for (sec, prop) in i.iter() {
        println!("Section: {:?}", sec);
        for (k, v) in prop.iter() {
            println!("{} ==> {}", k, v);
        }
    }

    for (_, prop) in i.iter_mut() {
        for (_, v) in prop.iter_mut() {
            if !(v.starts_with('"') && v.ends_with('"')) {
                let escaped = v.replace('"', "\\\"");
                *v = format!("\"{}\"", escaped);
            }
        }
    }

    i.write_to_file(path).unwrap();
    println!("saved: {}", path);
}
