# Project Amanda
Project Amanda is maze solver implemented for purposes of the "Parallel and Distributed Algorithms and Languages" course.

## Description
Amanda is trapped in the maze and she has to find the shortest possible exit to save herself. 

She has found the [map](src/lybrinth.bin) of the maze so she knows the coordinates of the exits, the positions of all doors, and the directions she can take.

Amanda decided to use the _A*_ algorithm and dispatch her _agents_ (threads) to each possible exit, letting them race.

She will follow the winning agent's path to survive the maze.






