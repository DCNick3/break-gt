package gametheory.assignment2.stratrnd2;

import gametheory.assignment2.Player;

import java.util.Random;

public class Strat implements Player {
  Random rnd = new Random();
  int ctr = 0;
  int move = rnd.nextInt(3) + 1;
  public void reset() {}
  public int move(int opponentLastMove, int xA, int xB, int xC) {
    ctr++;
    if (ctr >= 20) {
      ctr = 0;
      move = rnd.nextInt(2) + 2;
    }
    return move;
  }
  public String getEmail() { return ""; }
}
