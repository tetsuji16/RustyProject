package com.projectlibre.bridge;

import java.time.LocalDate;
import java.time.temporal.ChronoUnit;
import java.util.ArrayList;
import java.util.List;

final class TaskRecord {
    final int id;
    final String name;
    LocalDate start;
    LocalDate finish;
    double progress;
    final int indent;
    final boolean summary;
    final boolean milestone;
    final List<Integer> predecessors;

    TaskRecord(
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
        this.predecessors = new ArrayList<Integer>(predecessors);
    }

    long durationDays() {
        return milestone ? 0 : ChronoUnit.DAYS.between(start, finish) + 1;
    }
}
