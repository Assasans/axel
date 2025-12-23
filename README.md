<div align="center">

# Axel — a private server for *KonoSuba: Fantastic Days*

</div>

### Current progress

The first scene ("And so the Adventure Begins!") works.
The home and profile menus barely work.

A lot of gacha banners (including collaborations ones) is visible, but still not all 947 of them.
Loot pool is completely random, not saved to database.

All main quests are enabled. All stories (including collaboration ones) are enabled.
Battles work but no rewards are actually given, and you can't change your party.

All costumes and backgrounds are unlocked. Note that these customizations are not stored
on the server by client design, so they are not synced across accounts and devices.

Database support is being implemented, new account sequence works, you can change your name.

<p>
  <img src="https://files.catbox.moe/xvvt4z.png" width="300px">
  <img src="https://files.catbox.moe/x6a55m.png" width="300px">
  <img src="https://files.catbox.moe/ghnpk8.webp" width="300px">
  <img src="https://files.catbox.moe/jxc5gp.png" width="300px">
</p>

## Setup

See [CLIENT-SETUP.md](CLIENT-SETUP.md) for instructions on how to set up the client.

See [SERVER-SETUP.md](SERVER-SETUP.md) for instructions on how to set up the server and the database.

See [DUMPING.md](DUMPING.md) for instructions on how to inspect the game code yourself.

## Broken features

- In-game purchases (e.g. clicking add quartz button), hardlock, probably issue related to stub APK
- \[Home\] → \[Shop\], no server requests at all, hardlock, probably issue related to stub APK (\[Menu\] → \[Shop\] works)
- \[Others\] → \[Room Invitation\], unimplemented
- \[Quest\] → \[Event\] → \[Draw\], unimplemented
- \[Quest\] → \[Free Quest\], all items are locked
- \[Quest\] → \[Battle Arena\], unimplemented
- \[Quest\] → \[Dungeon\], unimplemented

Most working features are stubbed and do not save any progress.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or https://apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
