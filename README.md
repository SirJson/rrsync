# What is this?

This is a fork of rrsync, an rsync clone written in the Rust programming language. It is intended to provide the functionality of rsync, rdiff, and zsync in one single program, as well as some additions such as caching file signatures to make repeated synchronizations faster.

Basically, my story is: It is 2020, even Windows supports protocols like SSH natively and it is still hard to just host your own simple, secure, file sync between a master and a few slaves. I know the upstream owner of this project is working on an SSH backend for quite some time and still does. But I think what he is building something more universal while I'm trying to make it more opinionated to focus more on the problem I want to solve.

If everything works out like I hope it will, this will maybe no longer a tool that can do three things at a time, but more like a tool where any kind of device that can run the client can sync files with a master server in a secure way. Unfortunately, I plan on leaning on some features found on Linux Systems so I don't think there will be a Windows Master for now. But if it comes to clients, I want them to run as portable as possible. Not having great tools available on Windows is why this fork exists in the first place

# Current status

After I broke it and put it to gather I think the Core functionality is still there. You can index and sync local folders. I also got a grip on how this program is working at the moment I think.

The next (and final?) step is implementing the SSH transport to the master.

# Disclaimer

There is a real chance that I will remove features that wouldn't make sense anymore.

There is a even greater chance of me understanding why implementing an SSH Backend is so hard and at that point I have to think about my strategy again.

# How to use (for now)

Common options: `-X` indicates the location of the index file on the source side, and `-x` the index file on the destination side.

## rsync

```
$ rrsync sync some/folder othermachine:folder
```

Pre-computed indices are optional but make the operation faster:

```
$ rrsync index -x folder.idx some/folder
$ ssh othermachine \
  rrsync index -x folder.idx folder
$ rrsync sync -X folder.idx -x othermachine:folder.idx some/folder othermachine:folder
```

## rdiff

```bash
# Same as rdiff (signature/delta/patch)
$ rrsync index -x signature.idx old/folder
$ rrsync diff -o patch.bin -x signature.idx new/folder
$ rrsync patch old/folder patch.bin
```

## zsync

```bash
$ rrsync index -x data.tar.rrsync.idx data.tar
$ rrsync sync -X data.tar.rrsync.idx old/data.tar
# Or over network
$ rrsync sync -X http://example.org/data.tar.rrsync.idx old/data.tar
```

# Notes

The rsync algorithm: https://rsync.samba.org/tech_report/
How rsync works: https://rsync.samba.org/how-rsync-works.html

zsync: http://zsync.moria.org.uk/

Compression crate: https://crates.io/crates/flate2

Upstream Project: https://github.com/remram44/rrsync

And of course, thank you, remram44 for building these types of programs! I don't want to know what kind of voodoo I would have had to do if I wanted to get librsync to build on Windows.
