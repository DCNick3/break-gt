package gametheory.assignment2;
public interface Player {
  void reset();
  int move(int opponentLastMove, int xA, int xB, int xC);
  String getEmail();
}
