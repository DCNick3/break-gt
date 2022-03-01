package gametheory.assignment2.strat2;

import gametheory.assignment2.Player;

public class Strat implements Player {
  public void reset() {}
  public int move(int opponentLastMove, int xA, int xB, int xC) { return 2; }
  public String getEmail() { return ""; }
}
