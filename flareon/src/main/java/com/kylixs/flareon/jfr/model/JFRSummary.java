package com.kylixs.flareon.jfr.model;

import lombok.Data;

import java.util.Map;
import java.util.TreeMap;

/**
 * Created by Gong Dewei(kylixs) on 2022/12/11.
 */
@Data
public class JFRSummary {

    private long startTime;
    private long endTime;
    private long durationMs;
    private Map<String, Integer> eventStats = new TreeMap<>();

}
