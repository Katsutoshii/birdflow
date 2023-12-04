# Bevy Boids game prototype

Just messing around with some boids.

## Creatures

- Zooid
  - Separation and alignment with other zooids on the same team.
  - Attracted to the nearest ZooidHead, but less so than the nearest waypoint
- ZooidHead
  - All zooids follow this.
- Plankton
  - Food!
  - Moves around randomly
  - Zooids automatically chase nearby food

Two types of objects:
Dynamic: Move around and are affected by other objects.
Static: Aren't affected by other objects.

Dynamic objects have lots of shared behavior, e.g. chasing and repelling with each other.
But what about unique interactions?
Should they be in a different system or just a match case in the update?