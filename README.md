# Sudoku solver

A sudoku solver implementated in rust using constant propagation based on [this writeup by Peter Norvig](http://norvig.com/sudoku.html) with some performance optimizations.

## How to use it

Like any rust program, you can compile it, and then execute it or just `cargo run` it.

The program will read sudoku boards separated by empty lines from stdin and print the solutions to stdout.

The input format only cares about digits ([0-9]) or dots (`.`). Both a zero or a dot represent an unknown value to be solved.

For example, all these strings represent the same puzzle:

```4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......```

```
400000805
030000000
000700000
020000060
000080400
000010000
000603070
500200000
104000000
```

```
4 . . |. . . |8 . 5
. 3 . |. . . |. . .
. . . |7 . . |. . .
------+------+------
. 2 . |. . . |. 6 .
. . . |. 8 . |4 . .
. . . |. 1 . |. . .
------+------+------
. . . |6 . 3 |. 7 .
5 . . |2 . . |. . .
1 . 4 |. . . |. . .
```

### Execution example

```
$  echo -e "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......\n\n" | cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/sudoku_solver`
+------+------+------+
|4 . . |. . . |8 . 5 |
|. 3 . |. . . |. . . |
|. . . |7 . . |. . . |
+------+------+------+
|. 2 . |. . . |. 6 . |
|. . . |. 8 . |4 . . |
|. . . |. 1 . |. . . |
+------+------+------+
|. . . |6 . 3 |. 7 . |
|5 . . |2 . . |. . . |
|1 . 4 |. . . |. . . |
+------+------+------+

+------+------+------+
|4 1 7 |3 6 9 |8 2 5 |
|6 3 2 |1 5 8 |9 4 7 |
|9 5 8 |7 2 4 |3 1 6 |
+------+------+------+
|8 2 5 |4 3 7 |1 6 9 |
|7 9 1 |5 8 6 |4 3 2 |
|3 4 6 |9 1 2 |7 5 8 |
+------+------+------+
|2 8 9 |6 4 3 |5 7 1 |
|5 7 3 |2 9 1 |6 8 4 |
|1 6 4 |8 7 5 |2 9 3 |
+------+------+------+

(0.053638 seconds)
```
