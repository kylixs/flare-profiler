package com.kylixs.flareon.jfr;

import com.kylixs.flareon.jfr.model.JFRFileItem;
import com.kylixs.flareon.jfr.model.JFRSummary;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.Optional;

/**
 * Created by Gong Dewei(kylixs) on 2022/12/11.
 */
@RestController
@RequestMapping("/jfr")
public class JFRController {

    @Autowired
    private JFRService jfrService;

    @RequestMapping("/list")
    public List<JFRFileItem> list() {
        return jfrService.list();
    }

    @RequestMapping("/summary/{fileId}")
    public ResponseEntity<JFRSummary> summary(@PathVariable String fileId) {
        Optional<JFRSummary> summary = jfrService.getSummary(fileId);
        if (summary.isEmpty()) {
            return new ResponseEntity("JFR not found: " + fileId, HttpStatus.NOT_FOUND);
        }
        return new ResponseEntity<>(summary.get(), HttpStatus.OK);
    }


}
