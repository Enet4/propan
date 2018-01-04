# Propan

[![Build Status](https://travis-ci.org/Enet4/propan.svg?branch=master)](https://travis-ci.org/Enet4/propan)

Propan is a game. Likely an odd one.

![](https://img.itch.zone/aW1hZ2UvMTk4OTAyLzkzMzUwMi5wbmc=/347x500/uasNwE.png)

[View on itch.io](https://e-net4.itch.io/propan)

## How did this game come to life?

Propan was spiritually influenced by the game Helious from the early 90's. An urban legend tells that the game's creator, Sean Puckett, is not the original developer of the game, but that a UFO visited his house and the game was transferred to his computer by aliens during a stormy night. This enabled the peculiar story to be subtitled "How to Produce a Game in Eight Days." 

Considering the nature of this game, and the opening of the [GitHub Game Off 2017](https://itch.io/jam/game-off-2017) event, I took the challenge of creating
some sort of spin-off to Helious, by allocating my little spare time available to development. The first day, I got the project and the game engine set up. On the second day, I had a moving ball on the screen. And things went going until the 8th day. Just after the same number of days that Helious claimed to be produced in, I got very ill, and became incapable of working on the game for a whole week. The few days remaining, after the recovery and before the deadline, were dedicated to sprinting the necessary parts to have just enough for a "game-like" experience.

So anyway, I open-sourced everything under a permissive license. Hopefully the community will learn a bit from the code and give something back. It should be safe to say that no aliens were involved in interfering with this work. Or at least I think it is.

And yes, it's a Monet in the title screen. It seemed appropriate.

## Building

Propan is made in Rust, and can be built with the latest stable toolchains.

```sh
cargo run --release
```

## Playing

Once in the main menu, select a game level with the arrow keys on your keyboard.

While in the game, use the directional keys (or the keypad numbers 2, 4, 6, and 8) to move the ball by applying accelerations in those directions. The objective of each level is to collect all gems and touch the finish flag.

### Things to look out for

 - Your ball is inert and will not stop unless you apply a force in the opposite direction.
 - As you apply thrusts on the ball, it will slowly shrink. If the ball gets too small, it will implode.
 - Yellow wheels are pumps which can put your ball back in shape. Be careul though: too much pumping, and the ball will explode.
 - Stay away from mines. They will hurt you badly.
 - You can exit the level at any time by pressing the Escape button.
 - There is a level editor, which can be accessed by pressing "Shift + E", or by running the program with the subcommand `editor`.

## Using the level editor

You can enter the level editor by pressing "Shift + E", or by running the program with the subcommand `editor`.

```sh
propan editor
```

At the moment, loading a level requires the command line as well:

```sh
propan editor levels/3.json
```

In this mode, you have to use the mouse.

- *left mouse button* to place the currently defined object;
- *right mouse button* to delete an object in that position;
- click and drag the *middle mouse button* (mouse wheel) to move the camera;
- Roll the mouse wheel to select other items (wall, gem, pump, etc.);
- Press `,` (comma) and `.` (period) on your keyboard to choose a different wall texture (it will affect the wall's size);
- Press `S` on your keyboard to save the level into a new file.

Levels are saved in JSON, under a schema that should be fairly easy to understand. Although tedious, editing the game level by hand is possible, and is currently the only way to set the level's name.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

