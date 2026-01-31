# Modularitea libs

this is a library of [github.com/tealinuxos/tealinux-modularity](github.com/tealinuxos/tealinux-modularity) that give you headless way to testing or implement in on other stuff (GUI, CLI?)

# flow of calling

this libs contains some of subsystem

1. uses root privilege
2. non root

for root privileges, we need polkit in order to get permission from users, we can't trick this (for example, by using fork() and run standalone thread here, operating system will not allow this). instead use polkit such what normal GTK program do.

for program with root privilege needed (such changing grub theme, running system update, changing system parameters (zstd, swap size, etc)) we will pack them together on `./src/bin/program_name`. so in modularity perspective, we just need to do this


```sh
pkexec ls
```

here is critical features

- update system (require pacman)
- grup theme changer (require grub-mkconfig)
- install profile (require pacman)
- settings (WIP, this will contains a settings that change how tealinux behave, such ulimit, enable AI (yes/no), etc)

for settings itself is not implemented yet, because change enable AI yes/no can be considered require systemd (if run as daemon)
