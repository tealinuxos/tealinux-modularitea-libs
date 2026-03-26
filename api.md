# API's documentation

TLDR; this API contains 2 function, the first one is news, second one is grub changer. here i'll show you how to use it.

## news

this api allow you to fetch news from popular sites, (this is just a rss parser). any data that returned in this method is cached as *long as 1 hours*, another method to force refresh that cache is still in WIP.

so, what you can do right now is just call that methods and you'll get a bunch of data. here example of api call

```rust
use modularitea_libs::infrastructure::news_parser::NewsParser;

fn main() {
    let parser = match NewsParser::new() {
        Ok(parser) => parser,
        Err(err) => {
            eprintln!("failed to init parser: {err}");
            return;
        }
    };

    let items = match parser.blackbox_fetcher() {
        Ok(items) => items,
        Err(err) => {
            eprintln!("failed to fetch feeds: {err}");
            return;
        }
    };

    println!("{:#?}", items);
}
```

here the example output (truncated)

```txt
]
  ParsedNewsItem {
        url: "https://itsfoss.com/news/react-foundation-launch/",
        title: "React Is No Longer Meta&#x27;s Project, It Now Has Its Own Foundation",
        descriptive: "Meta has contributed React, React Native, and JSX to the newly formed React Foundation.",
        thumbnail: Some(
            "https://itsfoss.com/content/images/2026/02/react-foundation-banner.png",
        ),
    },
    ParsedNewsItem {
        url: "https://itsfoss.com/news/fedora-pocketblue-remix-overview/",
        title: "Someone is Bringing Fedora Linux to Phones (And It’s Not Red Hat)",
        descriptive: "A small community project is building immutable Fedora images for tablets and phones.",
        thumbnail: Some(
            "https://itsfoss.com/content/images/2026/02/fedora-pocketblue-remix-placeholder-banner.png",
        ),
    },
    ParsedNewsItem {
        url: "https://itsfoss.com/news/azul-malware-analysis-repository/",
        title: "Australia&#x27;s Cyber Agency Releases Azul, an Open Source Malware Analysis Repository",
        descriptive: "Think of it as a searchable, automated knowledge base for malware.",
        thumbnail: Some(
            "https://itsfoss.com/content/images/2026/02/azul-malware-analysis-repo-banner.png",
        ),
    },
    ParsedNewsItem {
        url: "https://itsfoss.com/modern-synaptic-style-package-manager/",
        title: "Making the Case for a Modern Synaptic-Style Package Manager on Linux",
        descriptive: "The Linux desktop has evolved, and it&#x27;s high time for the “advanced package manager” experience to evolve with it.",
        thumbnail: Some(
            "https://itsfoss.com/content/images/size/w1200/2026/02/synaptic-modern.webp",
        ),
    },
    ParsedNewsItem {
        url: "https://itsfoss.com/news/ladybird-web-browser-rustification/",
        title: "Ladybird Browser Just Ported C++ Code to Rust in 2 Weeks Thanks to AI",
        descriptive: "Turns out AI is pretty handy when you need to port C++ code.",
        thumbnail: Some(
            "https://itsfoss.com/content/images/2026/02/ladybird-browser-rustification-banner.png",
        ),
    },
    ParsedNewsItem {
        url: "https://itsfoss.com/linux-mint-keyboard-shortcuts/",
        title: "Master the Essential Keyboard Shortcuts in Linux Mint to Feel Like a Pro User",
        descriptive: "I am sharing some essential shortcuts to get you started. I will also briefly share how you can set custom shortcuts for all actions.",
        thumbnail: Some(
            "https://itsfoss.com/content/images/2026/02/linux-mint-shortcuts.webp",
        ),
    },
    ParsedNewsItem {
        url: "https://itsfoss.com/openclaw-alternatives/",
        title: "OpenClaw Alternatives That You Can Run on Raspberry Pi Like Devices",
        descriptive: "You don&#x27;t need a MacMini for running OpenClaw. These alternative projects can run on SBCs and ESP32 microcontrollers.",
        thumbnail: Some(
            "https://itsfoss.com/content/images/2026/02/openclaw-alternatives.webp",
        ),
    },
    ParsedNewsItem {
        url: "https://itsfoss.com/news/colorado-age-attestation-bill/",
        title: "US State Colorado Wants Operating Systems (Including Linux) to Tell Every App How Old You Are",
        descriptive: "A bill with a goal and vague language on how to achieve it.",
        thumbnail: Some(
            "https://itsfoss.com/content/images/2026/02/colorado-age-attestation-bill-banner.png",
        ),
    },
]
```


# grub changer

this feature is was ready to implement, here example API call

first, creating a context prelude


```rust
    let ret = GrubInstruction::with_themes_dir(
        "/home/fadhil_riyanto_guest/BALI64/tealinux-modularitea-libs/data/grub-theme".to_string(),
    )
    .set_screen_resolution(1920, 1080);
```

note that `set_screen_resolution` is madatory, fill `with_themes_dir` with location of theme pack (which ./data/grub-theme) in this repo.

SUGGEST: you could place a file-of `./data/grub-theme` in `/opt` or somewhere.

## get lists of all themes available

use this API to get all themes available.

call

```rust
let retq = ret.get_all_theme_available();
```

example output

```txt
]
  ThemeManifest {
        name: "doraemon",
        version: "1.0",
        github_url: Some(
            "https://github.com/MrVivekRajan/Grub-Themes",
        ),
        preview_image: Some(
            "",
        ),
        description: Some(
            "",
        ),
        author: Some(
            "MrVivekRajan",
        ),
        name_concat: None,
        steps: [],
    },
    ThemeManifest {
        name: "lorem-loader",
        version: "1.0",
        github_url: Some(
            "github.com/tealinuxos/lorem-loader/",
        ),
        preview_image: Some(
            "https://github.com/tealinuxos/lorem-loader/raw/master/assets/rule_of_thumb.png",
        ),
        description: Some(
            "Default grub theme for TealinuxOS",
        ),
        author: Some(
            "Fadhil Firmansyah",
        ),
        name_concat: None,
        steps: [],
    },
    ThemeManifest {
        name: "minegrub",
        version: "1.0",
        github_url: Some(
            "https://github.com/Lxtharia/minegrub-theme",
        ),
        preview_image: Some(
            "https://raw.githubusercontent.com/Lxtharia/minegrub-theme/dev/resources/preview_minegrub.png",
        ),
        description: Some(
            "A Minecraft-inspired GRUB theme with pixel art and blocky design elements.",
        ),
        author: Some(
            "Lxtharia",
        ),
        name_concat: None,
        steps: [],
    },
]
```

## applying theme

CAVEAT: DO NOT USE THIS METHOD IN RAW FORM, INSTEAD USE PREBUILD BINARIES ON `./target/release/modularitea-grub <theme_dir__hardcoded_before_call_pkexec> <theme_name>` to apply the changes. this allow you to safetly use pkexec btw.

the binary itself was available to be install using pacman

```rust
let _ = ret.apply_grub_theme(&theme);
```

where `theme` is a string theme name [ref](#get-lists-of-all-themes-available)

example output [https://gist.githubusercontent.com/fadhil-riyanto/c837808d90d3e8f2e25304a55135b6b9/raw/fab91dc4c4469bbbbbacd83aa14bb964dbd76dd5/Mon%2520Mar%2520%25202%252002:54:03%2520UTC%25202026](https://gist.githubusercontent.com/fadhil-riyanto/c837808d90d3e8f2e25304a55135b6b9/raw/fab91dc4c4469bbbbbacd83aa14bb964dbd76dd5/Mon%2520Mar%2520%25202%252002:54:03%2520UTC%25202026)



Last edited: Thu Mar 26 11:35:22 AM WIB 2026 by Fadhil Riyanto 