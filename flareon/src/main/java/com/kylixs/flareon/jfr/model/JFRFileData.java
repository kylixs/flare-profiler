package com.kylixs.flareon.jfr.model;

import lombok.Data;
import org.openjdk.jmc.common.item.IType;

import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Created by Gong Dewei(kylixs) on 2022/12/11.
 */
@Data
public class JFRFileData {

    private JFRFileItem file;

    private Map<IType, List<Object>> events = new ConcurrentHashMap<>();

    private long startTime;
    private long endTime;
    private long durationMs;

    public JFRFileData(JFRFileItem fileItem) {
        this.file = fileItem;
    }
}
