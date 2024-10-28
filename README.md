# Five Night's At Tellers 👻🐗
A scary game where my dog hunts me 

Play it [here](https://bradeneverson.github.io/five-nights-at-tellers/)

## Who Is Teller?
Teller is my dog. He's very nice most of the time but does have a knack for stalking me across hallways and sneaking up to attack. That's what inspired this Five Night's At Freddy's style game!

<img style="margin: 0px" height="300px" alt="Repository Header Image" src="https://github.com/user-attachments/assets/db9c2d5e-f0a7-4ee2-aeb7-c8e7f3493bb5" />
<-- Teller
<hr/>
Five Night's At Teller's is a WASM-backed game with all state being managed in Rust through the `wasm-pack` family of toolings. The frontend essentially is just communicating with this state and rendering appropriate images and sound effects when desired effects have to come up.

## Rules:

The rules of the game are relatively straightforward: don't let any of my dogs get into your office! To do so, you can view where the characters are positioned on a map relative to you by using the complementary security cameras. If anybody is directly outside of your room, you may want to close the door corresponding to that area. Keeping doors closed and surfing the cameras come at the cost of additional power consumption, however, and may cause you to lose all energy before the end of the night, leaving you defenseless.

## Stuff I Enjoy about Designing the Code:
* **Modular Enemy AI**: Enemy behavior is defined by a trait `EnemyBehavior` that has a single `tick` function and returns a Vector of actions that enemy will take in it's turn. The `tick` function has access to a reference to the current state, meaning it can use any context it wants to make complex decisions. The game state itself is then only aware of a non-owned Vector of trait objects allowing every enemy to behave differently. A few examples of current trait implementations that exist for different enemies include a pathfinding behavior that attempts to go straight to the player, a randomized behavior that just picks a random room to wander into, and a delayed "double moving" behavior that takes longer to perform actions but does 2 actions sequentially when it does.

  
* **Graph-based Map Generation and Travel**: The entire office layout is generated procedurally as a graph of Room nodes. These nodes begin at the security office as root, and diverge through a left and right hallway, this allows for graph traversal as a means of pathfinding, and easy frontend visaulization of the map as a series of room nodes and hallway connections, all in all a super cool use of graphs!
