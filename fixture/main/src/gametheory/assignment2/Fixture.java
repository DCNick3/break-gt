package gametheory.assignment2;

import java.io.BufferedReader;
import java.io.File;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.lang.reflect.Constructor;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.*;
import java.util.function.Supplier;
import java.util.stream.Collectors;

public class Fixture {
    private static final String PACKAGE_NAME = "gametheory.assignment2";
    private static final int MAX_MOVES = 100;

    private Set<Class<?>> findAllClassesInPackage(String packag3) {
        InputStream stream = ClassLoader.getSystemClassLoader()
                .getResourceAsStream(packag3.replaceAll("[.]", "/"));
        if (stream == null)
            return new HashSet<>();
        BufferedReader reader = new BufferedReader(new InputStreamReader(stream));
        return reader.lines()
                .filter(line -> line.endsWith(".class"))
                .map(n -> getClass(n, packag3))
                .collect(Collectors.toSet());
    }

    private Set<Class<?>> findAllClassesUsingClassLoader(String[] packages) {
        return Arrays.stream(packages)
                .map(r -> findAllClassesInPackage(PACKAGE_NAME + "." + r))
                .reduce(new HashSet<>(), (a, e) -> { a.addAll(e); return a; });
    }

    static Class<?> getClass(String className, String packag3) {
        try {
            return Class.forName(packag3 + "."
                    + className.substring(0, className.lastIndexOf('.')));
        } catch (ClassNotFoundException e) {

            // handle the exception
        }
        return null;
    }

    private Supplier<PlayerWrapper>[] getPlayers(String[] packages) {
        Set<Class<?>> classes = findAllClassesUsingClassLoader(packages);
        Class<?> playerInterface = Player.class;

        Supplier<?>[] suppliers = classes.stream().filter(clazz -> Arrays.stream(clazz.getInterfaces()).anyMatch(p -> p == playerInterface))
                .map(clazz -> {
                    Constructor<?> constructor;
                    try {
                        constructor = clazz.getConstructor();
                    } catch (NoSuchMethodException e) {
                        System.err.println("Cannot find constructor for player class " + clazz);
                        throw new RuntimeException(e);
                    }

                    return (Supplier<PlayerWrapper>) () -> {
                        Player player;
                        try {
                            player = (Player) constructor.newInstance();
                        } catch (Throwable e) {
                            System.err.println("Cannot construct player class " + clazz);
                            throw new RuntimeException(e);
                        }

                        String name = clazz.getCanonicalName();

                        return new PlayerWrapper(name, player);
                    };
                }).toArray(Supplier[]::new);
        return (Supplier<PlayerWrapper>[])suppliers;
    }

    class PlayerWrapper {
        public Player player;
        public String name;

        public PlayerWrapper(String name, Player p) {
            this.player = p;
            this.name = name;
        }

        void reset(MatchPlayerContext mc) {
            Future<?> fut = executor.submit(() -> this.player.reset());
            try {
                fut.get(100, TimeUnit.MILLISECONDS);
            } catch (TimeoutException ex) {
                mc.error = "timeout while executing reset()";
            } catch (Throwable e) {
                mc.error = "exception while executing reset():\n" + e;
            }
        }

        int getMove(MatchPlayerContext mc, int opponentLastMove, int[] x) {
            Future<?> fut = executor.submit(() -> this.player.move(opponentLastMove, x[0], x[1], x[2]));
            int res = -1;
            try {
                int res1 = (Integer) fut.get(100, TimeUnit.MILLISECONDS);
                if (res1 < 1 || res1 > 3) {
                    mc.error = "move out of bounds";
                } else {
                    res = res1;
                }
            } catch (TimeoutException ex) {
                mc.error = "timeout while executing move()";
            } catch (Throwable e) {
                mc.error = "exception while executing move():\n" + e;
            }

            mc.moves.add(res);

            return res;
        }
    }

    static String jsonStr(String s) {
        if (s == null)
            return "null";
        return "\"" + s.replace("\\", "\\\\")
                .replace("\t", "\\t")
                .replace("\b", "\\b")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\f", "\\f")
                .replace("'", "\\'")
                .replace("\"", "\\\"") + "\"";
    }

    static class MatchPlayerContext {
        public String playerName;
        public String error = null;
        public ArrayList<Integer> moves = new ArrayList<Integer>();
        public double score;

        public MatchPlayerContext(PlayerWrapper p1) {
            playerName = p1.name;
            score = 0;
        }

        public String serialize() {
            return "{    \n" +
                    "      \"player_name\": " + jsonStr(playerName) + ",\n" +
                    "      \"error\": " + jsonStr(error) + ",\n" +
                    "      \"score\": " + score + ",\n" +
                    "      \"moves\": [" + String.join(", ", moves.stream().map(Object::toString).collect(Collectors.toList())) + "]\n" +
                    "    }";
        }
    }

    static class MatchResult {
        int moves;
        MatchPlayerContext player1;
        MatchPlayerContext player2;

        public String serialize() {
            return "{  \n" +
                    "    \"moves\": " + moves + ",\n" +
                    "    \"player1\": " + player1.serialize() + ",\n" +
                    "    \"player2\": " + player2.serialize() + "\n" +
                    "  }";
        }
    }

    private String serializeMatchResults(MatchResult[] results) {
        return "[" + String.join(",\n", Arrays.stream(results).map(MatchResult::serialize).collect(Collectors.toList())) + "\n]";
    }

    private final ExecutorService executor = Executors.newFixedThreadPool(Runtime.getRuntime().availableProcessors() * 2, new DaemonThreadFactory());

    static class DaemonThreadFactory implements ThreadFactory {
        public Thread newThread(Runnable r) {
            Thread thread = new Thread(r);
            thread.setDaemon(true);
            return thread;
        }
    }

    static double f(int x) {
        return 10 * Math.exp(x) / (1 + Math.exp(x));
    }

    class Match {
        PlayerWrapper p1, p2;
        MatchPlayerContext ctx1, ctx2;

        public Match(PlayerWrapper p1, PlayerWrapper p2) {
            this.p1 = p1;
            this.p2 = p2;
            ctx1 = new MatchPlayerContext(p1);
            ctx2 = new MatchPlayerContext(p2);
        }

        boolean error() {
            return ctx1.error != null || ctx2.error != null;
        }

        MatchResult result(int moves) {
            MatchResult r = new MatchResult();
            r.moves = moves;
            r.player1 = ctx1;
            r.player2 = ctx2;
            return r;
        }

        MatchResult play() {
            p1.reset(ctx1);
            p2.reset(ctx2);
            if (error())
                return result(0);

            int p1Move = 0, p2Move = 0;

            int[] fields = new int[3];
            fields[0] = 1;
            fields[1] = 1;
            fields[2] = 1;

            for (int i = 0; i < MAX_MOVES; i++) {
                int p1NewMove = p1.getMove(ctx1, p2Move, fields);
                int p2NewMove = p2.getMove(ctx2, p1Move, fields);

                if (error())
                    return result(i);

                p1Move = p1NewMove;
                p2Move = p2NewMove;

                if (p1Move != p2Move) {
                    ctx1.score += f(fields[p1Move-1]) - f(0);
                    ctx2.score += f(fields[p2Move-1]) - f(0);
                }

                for (int p = 0; p < 3; p++) {
                    int dx;
                    if (p == p1Move - 1 || p == p2Move - 1)
                        dx = -1;
                    else
                        dx = 1;
                    if (fields[p] + dx >= 0)
                        fields[p] += dx;
                }
            }

            return result(MAX_MOVES);
        }
    }

    MatchResult playMatch(PlayerWrapper p1, PlayerWrapper p2) {
        return new Match(p1, p2).play();
    }

    void realMain(String[] args) {
        File classFile = new File(Fixture.class.getProtectionDomain().getCodeSource().getLocation().getPath() + "/"
                + PACKAGE_NAME.replace(".", "/"));

        String[] packages = Arrays.stream(classFile.listFiles()).filter(f -> f.isDirectory()).map(f -> f.getName()).toArray(String[]::new);

//        String[] packages = new String[] {
//                "strat1",
//                "strat2",
//                "stratmirror",
//                "stratrnd",
//                "stratrnd2",
//
//        };

        Supplier<PlayerWrapper>[] players = getPlayers(packages);

        ArrayList<Match> matches = new ArrayList<>();
        for (int i = 1; i < players.length; i++)
            for (int j = i; ++j <= players.length;)
            {
                matches.add(new Match(players[i-1].get(), players[j-1].get()));
            }
        MatchResult[] results = matches.parallelStream()
                .map(Match::play)
                .toArray(MatchResult[]::new);

        String s = serializeMatchResults(results);

        s = s.replace("\n", "");

        System.out.println(s);
    }

    public static void main(String[] args) {
        new Fixture().realMain(args);
    }
}
