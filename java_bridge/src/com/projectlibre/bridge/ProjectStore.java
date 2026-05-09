package com.projectlibre.bridge;

import java.time.LocalDate;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

final class ProjectStore {
    private final List<TaskRecord> tasks;

    private ProjectStore(List<TaskRecord> tasks) {
        this.tasks = tasks;
    }

    static ProjectStore sample() {
        List<TaskRecord> tasks = new ArrayList<TaskRecord>();
        tasks.add(task(32, "Construction", "2025-01-29", "2025-03-21", 0.43, 0, true, false));
        tasks.add(task(33, "Site preparation", "2025-01-29", "2025-02-06", 1.00, 0, true, false));
        tasks.add(task(34, "Obtain permits", "2025-01-29", "2025-01-31", 1.00, 1, false, false));
        tasks.add(task(35, "Survey and stake building", "2025-02-03", "2025-02-03", 1.00, 1, false, false, 34));
        tasks.add(task(36, "Clear lot", "2025-02-03", "2025-02-04", 1.00, 1, false, false, 35));
        tasks.add(task(37, "Temporary utilities", "2025-02-05", "2025-02-06", 0.80, 1, false, false, 36));
        tasks.add(task(38, "Site ready milestone", "2025-02-06", "2025-02-06", 1.00, 1, false, true, 37));
        tasks.add(task(39, "Foundation", "2025-02-07", "2025-02-20", 0.62, 0, true, false, 38));
        tasks.add(task(40, "Excavate footings", "2025-02-07", "2025-02-10", 1.00, 1, false, false, 38));
        tasks.add(task(41, "Pour concrete footings", "2025-02-11", "2025-02-14", 0.90, 1, false, false, 40));
        tasks.add(task(42, "Slab", "2025-02-17", "2025-02-20", 0.35, 0, true, false, 41));
        tasks.add(task(43, "Install vapor barrier", "2025-02-17", "2025-02-17", 0.70, 1, false, false, 41));
        tasks.add(task(44, "Pour slab", "2025-02-18", "2025-02-20", 0.20, 1, false, false, 43));
        tasks.add(task(45, "Framing", "2025-02-21", "2025-03-05", 0.21, 0, true, false, 44));
        tasks.add(task(46, "Frame exterior walls", "2025-02-21", "2025-02-26", 0.35, 1, false, false, 44));
        tasks.add(task(47, "Set roof trusses", "2025-02-27", "2025-03-05", 0.10, 1, false, false, 46));
        tasks.add(task(48, "Mechanical rough-in", "2025-03-03", "2025-03-11", 0.05, 0, true, false, 46));
        tasks.add(task(49, "Electrical rough-in", "2025-03-03", "2025-03-06", 0.10, 1, false, false, 46));
        tasks.add(task(50, "Plumbing rough-in", "2025-03-05", "2025-03-10", 0.00, 1, false, false, 49));
        tasks.add(task(51, "HVAC rough-in", "2025-03-07", "2025-03-11", 0.00, 1, false, false, 49));
        tasks.add(task(52, "Exterior close-in", "2025-03-06", "2025-03-17", 0.00, 0, true, false, 47));
        tasks.add(task(53, "Install roofing", "2025-03-06", "2025-03-10", 0.00, 1, false, false, 47));
        tasks.add(task(54, "Install windows", "2025-03-11", "2025-03-13", 0.00, 1, false, false, 53));
        tasks.add(task(55, "Exterior inspection", "2025-03-17", "2025-03-17", 0.00, 1, false, true, 54));
        tasks.add(task(56, "Interior finish", "2025-03-12", "2025-03-21", 0.00, 0, true, false, 51));
        tasks.add(task(57, "Insulation", "2025-03-12", "2025-03-13", 0.00, 1, false, false, 51));
        tasks.add(task(58, "Drywall", "2025-03-14", "2025-03-19", 0.00, 1, false, false, 57));
        tasks.add(task(59, "Final phase", "2025-03-20", "2025-03-21", 0.00, 0, true, false, 55, 58));
        tasks.add(task(60, "Punch list", "2025-03-20", "2025-03-21", 0.00, 1, false, false, 55, 58));
        tasks.add(task(61, "Substantial completion", "2025-03-21", "2025-03-21", 0.00, 1, false, true, 60));
        ProjectStore store = new ProjectStore(tasks);
        store.recomputeSummaries();
        return store;
    }

    void moveTask(int id, LocalDate start, LocalDate finish) {
        TaskRecord task = find(id);
        task.start = start;
        task.finish = finish;
        if (task.milestone) {
            task.finish = task.start;
        }
        recomputeSummaries();
    }

    void resizeStart(int id, LocalDate start) {
        TaskRecord task = find(id);
        task.start = start.isAfter(task.finish) ? task.finish : start;
        if (task.milestone) {
            task.finish = task.start;
        }
        recomputeSummaries();
    }

    void resizeEnd(int id, LocalDate finish) {
        TaskRecord task = find(id);
        task.finish = finish.isBefore(task.start) ? task.start : finish;
        if (task.milestone) {
            task.start = task.finish;
        }
        recomputeSummaries();
    }

    void setProgress(int id, double progress) {
        TaskRecord task = find(id);
        task.progress = clamp(progress, 0.0, 1.0);
        recomputeSummaries();
    }

    ProjectSnapshot snapshot() {
        recomputeSummaries();
        LocalDate start = tasks.get(0).start;
        LocalDate end = tasks.get(0).finish;
        List<TaskSnapshot> out = new ArrayList<TaskSnapshot>();
        for (TaskRecord task : tasks) {
            if (task.start.isBefore(start)) {
                start = task.start;
            }
            if (task.finish.isAfter(end)) {
                end = task.finish;
            }
            out.add(new TaskSnapshot(
                    task.id,
                    task.name,
                    task.start,
                    task.finish,
                    task.progress,
                    task.indent,
                    task.summary,
                    task.milestone,
                    new ArrayList<Integer>(task.predecessors)));
        }
        return new ProjectSnapshot(start, end, out);
    }

    private TaskRecord find(int id) {
        for (TaskRecord task : tasks) {
            if (task.id == id) {
                return task;
            }
        }
        throw new IllegalArgumentException("Unknown task id: " + id);
    }

    private void recomputeSummaries() {
        for (int i = tasks.size() - 1; i >= 0; i--) {
            TaskRecord task = tasks.get(i);
            if (!task.summary) {
                continue;
            }

            int indent = task.indent;
            List<TaskRecord> children = new ArrayList<TaskRecord>();
            for (int j = i + 1; j < tasks.size(); j++) {
                TaskRecord child = tasks.get(j);
                if (child.indent <= indent) {
                    break;
                }
                if (child.indent == indent + 1) {
                    children.add(child);
                }
            }

            if (children.isEmpty()) {
                continue;
            }

            LocalDate minStart = children.get(0).start;
            LocalDate maxFinish = children.get(0).finish;
            double weighted = 0.0;
            double total = 0.0;
            for (TaskRecord child : children) {
                if (child.start.isBefore(minStart)) {
                    minStart = child.start;
                }
                if (child.finish.isAfter(maxFinish)) {
                    maxFinish = child.finish;
                }
                double duration = Math.max(1L, child.durationDays());
                weighted += child.progress * duration;
                total += duration;
            }

            task.start = minStart;
            task.finish = maxFinish;
            task.progress = total == 0.0 ? 0.0 : weighted / total;
        }
    }

    private static TaskRecord task(
            int id,
            String name,
            String start,
            String finish,
            double progress,
            int indent,
            boolean summary,
            boolean milestone,
            Integer... predecessors) {
        return new TaskRecord(
                id,
                name,
                LocalDate.parse(start),
                LocalDate.parse(finish),
                progress,
                indent,
                summary,
                milestone,
                Arrays.asList(predecessors));
    }

    private static double clamp(double value, double min, double max) {
        return Math.max(min, Math.min(max, value));
    }
}

final class ProjectSnapshot {
    final LocalDate startDate;
    final LocalDate endDate;
    final List<TaskSnapshot> tasks;

    ProjectSnapshot(LocalDate startDate, LocalDate endDate, List<TaskSnapshot> tasks) {
        this.startDate = startDate;
        this.endDate = endDate;
        this.tasks = tasks;
    }
}

final class TaskSnapshot {
    final int id;
    final String name;
    final LocalDate start;
    final LocalDate finish;
    final double progress;
    final int indent;
    final boolean summary;
    final boolean milestone;
    final List<Integer> predecessors;

    TaskSnapshot(
            int id,
            String name,
            LocalDate start,
            LocalDate finish,
            double progress,
            int indent,
            boolean summary,
            boolean milestone,
            List<Integer> predecessors) {
        this.id = id;
        this.name = name;
        this.start = start;
        this.finish = finish;
        this.progress = progress;
        this.indent = indent;
        this.summary = summary;
        this.milestone = milestone;
        this.predecessors = predecessors;
    }
}
