package com.projectlibre.bridge;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.time.LocalDate;
import java.util.Base64;
import java.util.List;
import java.util.Locale;

public final class BridgeServer {
    public static void main(String[] args) throws Exception {
        ProjectStore store = ProjectStore.sample();
        BufferedReader input = new BufferedReader(
                new InputStreamReader(System.in, StandardCharsets.UTF_8));
        PrintWriter output = new PrintWriter(
                new OutputStreamWriter(System.out, StandardCharsets.UTF_8), true);

        String line;
        while ((line = input.readLine()) != null) {
            if (line.isEmpty()) {
                continue;
            }

            try {
                handle(store, line, output);
            } catch (Exception ex) {
                output.println("ERR\t" + encode(ex.getMessage() == null ? ex.toString() : ex.getMessage()));
                output.println("END");
            }
        }
    }

    private static void handle(ProjectStore store, String line, PrintWriter output) {
        String[] parts = line.split("\t", -1);
        String command = parts[0].toUpperCase(Locale.ROOT);

        if ("SNAPSHOT".equals(command)) {
            writeSnapshot(output, store.snapshot());
            return;
        }

        if ("MOVE_ABS".equals(command)) {
            store.moveTask(Integer.parseInt(parts[1]), LocalDate.parse(parts[2]), LocalDate.parse(parts[3]));
            writeSnapshot(output, store.snapshot());
            return;
        }

        if ("RESIZE_START_ABS".equals(command)) {
            store.resizeStart(Integer.parseInt(parts[1]), LocalDate.parse(parts[2]));
            writeSnapshot(output, store.snapshot());
            return;
        }

        if ("RESIZE_END_ABS".equals(command)) {
            store.resizeEnd(Integer.parseInt(parts[1]), LocalDate.parse(parts[2]));
            writeSnapshot(output, store.snapshot());
            return;
        }

        if ("SET_PROGRESS".equals(command)) {
            store.setProgress(Integer.parseInt(parts[1]), Double.parseDouble(parts[2]));
            writeSnapshot(output, store.snapshot());
            return;
        }

        throw new IllegalArgumentException("Unknown command: " + command);
    }

    private static void writeSnapshot(PrintWriter output, ProjectSnapshot snapshot) {
        output.println("OK");
        output.println("RANGE\t" + snapshot.startDate + "\t" + snapshot.endDate);
        output.println("TASKS\t" + snapshot.tasks.size());
        for (TaskSnapshot task : snapshot.tasks) {
            output.println(
                    "TASK\t"
                            + task.id
                            + "\t"
                            + encode(task.name)
                            + "\t"
                            + task.start
                            + "\t"
                            + task.finish
                            + "\t"
                            + String.format(Locale.ROOT, "%.4f", task.progress)
                            + "\t"
                            + task.indent
                            + "\t"
                            + task.summary
                            + "\t"
                            + task.milestone
                            + "\t"
                            + join(task.predecessors));
        }
        output.println("END");
    }

    private static String join(List<Integer> values) {
        if (values.isEmpty()) {
            return "";
        }

        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < values.size(); i++) {
            if (i > 0) {
                sb.append(',');
            }
            sb.append(values.get(i));
        }
        return sb.toString();
    }

    private static String encode(String value) {
        return Base64.getUrlEncoder().withoutPadding().encodeToString(value.getBytes(StandardCharsets.UTF_8));
    }
}
