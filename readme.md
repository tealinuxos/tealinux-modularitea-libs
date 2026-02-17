# Modularitea libs

this is a library of [github.com/tealinuxos/tealinux-modularity](github.com/tealinuxos/tealinux-modularity) that give you headless way to testing or implement in on other stuff (GUI, CLI?)

## list of feature (implemeted and not yet)

**HARD NOTE, THIS FEATURE BELOW IS LISTED AS EXTERNAL EXECUTABLE BECAUSE ITS NEED ROOT TO RUN**

if you developing a feature that doesn't need to run as root, do not list on this table, THIS feature is excluded (from) the libs because we need to run pkexec (on GUI side), so the users don't need to run tealinux-modularitea as root, instead the confirmation promp will appear when they try to executable spesific feature, which reflect to this package

the output of this program (listen on the table) is always json, please return the output (from stdin, or stderr, whatever) as json, this program is itended for programatically use, rather being executed by end-users

| feature name        | executable name               | status                          | comment                                                          |
|---------------------|-------------------------------|---------------------------------|------------------------------------------------------------------|
| profile installer   | modularitea-profile-installer | implemented, but in wrong place | allow to install spesific profile (a sets of package)            |
| grub changer        | modularitea-grub-changer      | unimplemented, untested         | allow users to change their grub theme                           |
| tools (enable swap) | modularitea-tools-enable-swap | unimplemented                   | allow users to enable swap (always ZSTD)                         |
| tools (enable AI)   | modularitea-tools-enable-ai   | unimplemented                   | allow users to enable AI feature (by configuring systemd config) |

## lists of non root feature

this feature is available with direct API calls (a func or methods), with no external executable needed

- news API, if you expert at parsing XML files, you can get the data from popular RSS system (dev.to, phorenix, github, whatever), its great place btw.
- system info (it was implemented perfectly, but wrong place, you should implement it on this lib instead on main program)

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
