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
