# KonoSuba: Fantastic Days - Reverse Engineering

This tutorial should apply to any LIAPP-encrypted IL2CPP Unity game.

## 1. Dumping `global-metadata.dat`

The metadata file is located at `assets/bin/Data/Managed/Metadata/global-metadata.dat`.
LIAPP-encrypted metadata file starts with `4C 49 4B 45 59` (`LIKEY`), we need to get a metadata file that starts with `AF 1B B1 FA`.

The easiest way to get decrypted metadata is to dump the memory of the running game.

1. Start `KonoSuba: FD` in Waydroid and wait for the title screen to appear.
2. Open `top` on the host machine and search for `konosuba`.
   There should be 2 processes — the actual game (~700 MB memory) and the watcher (1 thread).
3. *Note: If you now try to connect `gdb` to the main process, you will get `process <main> is already traced by process <watcher>` error.
   If you kill the watcher process, it will be immediately restarted.*
4. Run `sudo gdb --pid <pid of watcher process>` and make sure that `gdb` has successfully connected to the watcher process.
5. Return to `top` and send `SIGKILL` to the watcher process. It will become a zombie process because `gdb` is still connected.
   All its handles (including `ptrace`) will close, but the game won't restart it because it hasn't quit yet.
6. Run `sudo gdb --pid <pid of main process>`.
7. Enable logging to file — `set logging on`, log is stored in `gdb.txt` file.
8. Run `info proc mappings` -> `c`.
9. Open `gdb.txt` file in a text editor and search for `global-metadata.dat`. 
   You will get output similar to this:

```
0x97274000 0x97475000   0x201000        0x0  rw-p   [anon:libc_malloc]
0x97475000 0x982ea000   0xe75000        0x0  r--s   /storage/emulated/0/Android/data/com.nexon.konosuba/files/il2cpp/Metadata/global-metadata.dat
0x982ea000 0x994db000  0x11f1000        0x0  rw-p   [anon:libc_malloc]
0x994db000 0x9956d000    0x92000        0x0  rw-p   [anon:libc_malloc]
```

or

```
0x7be164600000     0x7be1654aa000   0xeaa000        0x0  r--s   /storage/emulated/0/Android/data/com.nexon.konosuba/files/il2cpp/Metadata/global-metadata.dat
0x7be1654ab000     0x7be1654ae000     0x3000        0x0  rw-p   [anon:Mem_0x10001000]
0x7be1654ae000     0x7be1654b2000     0x4000        0x0  rw-p   [anon:dalvik-indirect ref table]
...
0x7be1654ff000     0x7be165600000   0x101000        0x0  rw-p   [anon:Mem_0x10000004]
0x7be165600000     0x7be168200000  0x2c00000        0x0  rw-p   [anon:libc_malloc]
```

The `global-metadata.dat` region still contains encrypted data, so we ignore it.
We are interested in a large `rw-p [anon:libc_malloc]` region following it — in my case it was:

- `0x982ea000         0x994db000      0x11f1000  0x0  rw-p  [anon:libc_malloc]` for v4.5.11
- `0x7be165600000     0x7be168200000  0x2c00000  0x0  rw-p  [anon:libc_malloc]` for v4.11.2

It may not be immediately after

10. Dump it to a file — `dump binary memory dump.bin 0x7be165600000 0x7be168200000`.
11. Open the dumped file in HEX editor and search for the `global-metadata.dat` signature — `AF 1B B1 FA`, in my case it was at offset `0000:0AC0` for v4.5.11 and `0180:0D40` for v4.11.2.
12. Save everything starting at this offset to a new file — `global-metadata-decrypted.dat`. This is your decrypted `global-metadata.dat`.

## 2. Importing metadata into IDA Pro

1. Use [IL2CppDumper](https://github.com/Perfare/Il2CppDumper) to dump definitions.
2. Load the `libil2cpp.so` file into IDA Pro, **do not** run the analysis (uncheck "Analysis → Enable").
3. Load Python script (Alt + F7) `ida_with_struct_py3.py` and select your dumped `script.json` and `il2cpp.h` files.
4. Open the "Options → General → Analysis" window, check "Analysis → Enable" and click "Reanalyze program".
5. The analysis will take a long time (several hours), but you can start analyzing the code before it is finished.
   Remember to save the database regularly (Ctrl + W) to avoid losing your progress.

## Useful resources

[IL2CPP Tutorial: Finding loaders for obfuscated global-metadata.dat files](https://katyscode.wordpress.com/2021/02/23/il2cpp-finding-obfuscated-global-metadata/)
