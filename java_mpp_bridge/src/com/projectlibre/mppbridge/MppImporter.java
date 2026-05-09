package com.projectlibre.mppbridge;

import java.io.File;
import java.time.LocalDate;
import java.time.LocalDateTime;
import java.util.List;
import java.util.Locale;

import org.mpxj.ProjectFile;
import org.mpxj.Relation;
import org.mpxj.Task;
import org.mpxj.reader.UniversalProjectReader;

public final class MppImporter {
    private MppImporter() {
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            throw new IllegalArgumentException("Usage: MppImporter <file.mpp>");
        }

        ProjectFile project = new UniversalProjectReader().read(new File(args[0]));
        List<Task> tasks = project.getTasks();

        StringBuilder json = new StringBuilder();
        json.append('{');
        json.append("\"start_date\":");
        json.append(findProjectStart(tasks));
        json.append(',');
        json.append("\"end_date\":");
        json.append(findProjectEnd(tasks));
        json.append(',');
        json.append("\"tasks\":[");

        boolean firstTask = true;
        for (Task task : tasks) {
            if (task.getNull()) {
                continue;
            }
            if (!firstTask) {
                json.append(',');
            }
            firstTask = false;
            json.append('{');
            appendField(json, "id", valueOf(task.getUniqueID()));
            json.append(',');
            appendField(json, "name", quote(nullToEmpty(task.getName())));
            json.append(',');
            appendField(json, "start", formatOrDefault(dateValue(task.getStart(), task.getEarlyStart())));
            json.append(',');
            appendField(json, "finish", formatOrDefault(dateValue(task.getFinish(), task.getEarlyFinish())));
            json.append(',');
            appendField(json, "progress", String.valueOf(progress(task)));
            json.append(',');
            appendField(json, "indent", String.valueOf(Math.max(0, outlineLevel(task) - 1)));
            json.append(',');
            appendField(json, "summary", String.valueOf(booleanValue(task.getSummary())));
            json.append(',');
            appendField(json, "milestone", String.valueOf(booleanValue(task.getMilestone())));
            json.append(',');
            json.append("\"predecessors\":[");
            boolean firstPred = true;
            for (Relation relation : task.getPredecessors()) {
                Task pred = relation.getPredecessorTask();
                if (pred == null || pred.getUniqueID() == null) {
                    continue;
                }
                if (!firstPred) {
                    json.append(',');
                }
                firstPred = false;
                json.append(pred.getUniqueID().intValue());
            }
            json.append(']');
            json.append('}');
        }

        json.append("]}");
        System.out.println(json.toString());
    }

    private static String findProjectStart(List<Task> tasks) {
        LocalDate earliest = null;
        for (Task task : tasks) {
            LocalDate start = dateValue(task.getStart(), task.getEarlyStart());
            if (start != null && (earliest == null || start.isBefore(earliest))) {
                earliest = start;
            }
        }
        return formatOrDefault(earliest);
    }

    private static String findProjectEnd(List<Task> tasks) {
        LocalDate latest = null;
        for (Task task : tasks) {
            LocalDate finish = dateValue(task.getFinish(), task.getEarlyFinish());
            if (finish != null && (latest == null || finish.isAfter(latest))) {
                latest = finish;
            }
        }
        return formatOrDefault(latest);
    }

    private static LocalDate dateValue(LocalDateTime primary, LocalDateTime fallback) {
        LocalDateTime value = primary != null ? primary : fallback;
        return value == null ? null : value.toLocalDate();
    }

    private static double progress(Task task) {
        Number value = task.getPercentageComplete();
        if (value == null) {
            return 0.0;
        }
        return Math.max(0.0, Math.min(1.0, value.doubleValue() / 100.0));
    }

    private static int outlineLevel(Task task) {
        Integer value = task.getOutlineLevel();
        return value == null ? 1 : value.intValue();
    }

    private static boolean booleanValue(Boolean value) {
        return value != null && value.booleanValue();
    }

    private static String valueOf(Integer value) {
        return value == null ? "0" : String.valueOf(value.intValue());
    }

    private static String nullToEmpty(String value) {
        return value == null ? "" : value;
    }

    private static String formatOrDefault(LocalDate value) {
        return quote(value == null ? "2025-01-01" : value.toString());
    }

    private static String quote(String value) {
        return "\"" + escape(value) + "\"";
    }

    private static String escape(String value) {
        StringBuilder out = new StringBuilder();
        for (int i = 0; i < value.length(); i++) {
            char ch = value.charAt(i);
            switch (ch) {
                case '\\':
                    out.append("\\\\");
                    break;
                case '"':
                    out.append("\\\"");
                    break;
                case '\b':
                    out.append("\\b");
                    break;
                case '\f':
                    out.append("\\f");
                    break;
                case '\n':
                    out.append("\\n");
                    break;
                case '\r':
                    out.append("\\r");
                    break;
                case '\t':
                    out.append("\\t");
                    break;
                default:
                    if (ch < 0x20) {
                        out.append(String.format(Locale.ROOT, "\\u%04x", (int) ch));
                    } else {
                        out.append(ch);
                    }
                    break;
            }
        }
        return out.toString();
    }

    private static void appendField(StringBuilder json, String key, String value) {
        json.append('"').append(key).append('"').append(':').append(value);
    }
}
