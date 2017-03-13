# Scavenger of Broken Dreams
My submission for the 2017 7DRL Challenge. It doesn't have in game help because
I didn't get around to implementing it. Sorry about that. This README should
tell you everything you need to know.

You're in some sort of basin surrounded by cliffs. Your only way out is to read
your scroll of recall. The ground is littered with diamonds, and the corpses of
those who came before you. Maybe they died carrying something useful? Try to
collect as many diamonds as possible, but don't die in the process.

## Controls
Use arrow keys, vi keys, or numpad to move (the game uses 8-direction movement).
Space or '5' can be used to wait a turn. Special actions are as follows:

  - 'R': read your scroll of recall
  - 'e': eat a healing herb
  - 'g': pick up a corpse (all other items are automatically picked up)
  - 'd': drop a corpse
  - 'f': switch to fire arrow mode
  - 't': switch to throw rock mode
  - 'N': start a new game
  - 'Q' or ESC: quit the game

In fire arrow mode, you can use 'f', space, or '5' to exit the mode or a
directional key to fire an arrow in one of 8 directions.

In throw rock mode, you can use space, or '5' to exit the mode,
directional keys to target an enemy, and 't' to throw a rock at a selected
visible enemy.

## Items
Your inventory (and health) is displayed along the top of the screen. Each item
type is displayed with how many of them you have, or if it doesn't make sense to
have multiples it is light colored if you have one and darkened if you don't
have any. Item types are as follows:

  - white '?': Your scroll of recall (you start with one). Press 'R' to read it
    and be teleported to safety in 20-30 turns.
  - white '|': A sword. Increases your bump to attack damage from 1 to 3.
  - yellow '}': A bow. Required for firing arrows.
  - yellow '/': Arrows. If you have a bow, you can 'f'ire them to do 2 damage.
  - white '*': Rocks. Can be 't'hrown at enemies to do 1 damage.
  - red '%': Corpse. Drops when you kill enemies. Currently complete pointless.
  - green '+': Healing herbs. Heals 1 damage when 'e'aten.
  - cyan '*': Diamond. Try to get as many as possible without dying (the map
    contains 30).

## Creatures
These are the inhabitants of this place. You should mostly try to avoid them.

  - 'r': Rat. Hits for 1 damage and has 2 health.
  - 'd': Deer. Has 5 health and tries to avoid you. Harmless.
  - 'w': Wolf. Hits for 2 damage and has 5 health.
  - 'D': Dragon. Hits for 3 damage and has 15 health.

## Hints
Don't try to fight enemies, and don't get greedy. The enemy pathfinding is
hilariously bad, so you can generally lose them by ducking around trees.
