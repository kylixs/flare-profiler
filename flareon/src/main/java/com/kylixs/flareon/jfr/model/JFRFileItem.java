package com.kylixs.flareon.jfr.model;

import lombok.Data;

import java.util.Date;

/**
 * Created by Gong Dewei(kylixs) on 2022/12/11.
 */
@Data
public class JFRFileItem {
    private String id;
    private String name;
    private String path;
    private long size;
    private Date createdTime;


}
