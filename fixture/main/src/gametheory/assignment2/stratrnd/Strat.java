package gametheory.assignment2.stratrnd;

import gametheory.assignment2.Player;

import java.util.Random;

public class Strat implements Player {
  Random rnd = new Random();
  public void reset() {}
  public int move(int opponentLastMove, int xA, int xB, int xC) {
    return rnd.nextInt(3) + 1;
  }
  public String getEmail() { return ""; }
}
