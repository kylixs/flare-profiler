package com.kylixs.flareon.jfr;

import com.kylixs.flareon.jfr.model.JFRFileData;
import com.kylixs.flareon.jfr.model.JFRFileItem;
import com.kylixs.flareon.jfr.model.JFRSummary;
import lombok.extern.slf4j.Slf4j;
import org.openjdk.jmc.common.IDescribable;
import org.openjdk.jmc.common.item.IAccessorKey;
import org.openjdk.jmc.common.item.IItem;
import org.openjdk.jmc.common.item.IItemCollection;
import org.openjdk.jmc.common.item.IItemIterable;
import org.openjdk.jmc.common.item.IMemberAccessor;
import org.openjdk.jmc.common.item.IType;
import org.openjdk.jmc.common.item.ItemToolkit;
import org.openjdk.jmc.flightrecorder.JfrLoaderToolkit;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.io.File;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.zip.CRC32;

/**
 * Created by Gong Dewei(kylixs) on 2022/12/11.
 */
@Slf4j
@Service
public class JFRService {
    private String dataDirPath = "flareon/samples/jfr/";

    private List<JFRFileItem> fileItems = Collections.synchronizedList(new ArrayList<>());

    private Map<String, JFRFileData> fileCache = new ConcurrentHashMap<>();

    @PostConstruct
    public void init() {
        try {
            list();
        } catch (Exception e) {
            log.error("init jfr files failed: {}", e.getMessage(), e);
        }
    }

    public List<JFRFileItem> list() {
        File dataDir = new File(dataDirPath);
        File[] files = dataDir.listFiles(f -> f.isFile() && f.getName().endsWith(".jfr"));
        this.fileItems = Arrays.stream(files)
                .map(this::createJFRFile).toList();
        log.info("Loaded {} jfr files from data dir: {}", this.fileItems.size(), this.fileItems);
        return this.fileItems;
    }

    public Optional<JFRSummary> getSummary(String fileId) {
        Optional<JFRFileItem> fileItem = getFileItemById(fileId);
        if (fileItem.isEmpty()) {
            return Optional.empty();
        }

        long start = 0, end = 0;
        JFRFileData fileData = fileCache.computeIfAbsent(fileId, _k -> new JFRFileData(fileItem.get()));
        if (fileData.getEvents().isEmpty()) {
            try {
                IItemCollection events = JfrLoaderToolkit.loadEvents(Arrays.asList(new File(fileItem.get().getPath())));

                Iterator<IItemIterable> itemIterable = events.iterator();
                while (itemIterable.hasNext()) {
                    Iterator<IItem> itemIterator = itemIterable.next().iterator();
                    while (itemIterator.hasNext()) {
                        Map<String, Object> eventData = statsEvent(itemIterator.next(), fileData);
                        Number time = (Number) eventData.get("startTime");
                        if (time != null) {
                            if (start == 0) {
                                start = time.longValue();
                            }
                            end = time.longValue();
                        }
                    }
                }

            } catch (Exception e) {
                log.error("Parse jfr file failed: {}, file: {}", e.getMessage(), fileItem.get().getPath(), e);
            }

            fileData.setStartTime(start);
            fileData.setEndTime(end);
            fileData.setDurationMs((end - start) / 1000000);
        }

        JFRSummary summary = new JFRSummary();
        summary.setStartTime(fileData.getStartTime());
        summary.setEndTime(fileData.getEndTime());
        summary.setDurationMs(fileData.getDurationMs());
        fileData.getEvents().entrySet().forEach(entry -> {
            summary.getEventStats().put(entry.getKey().getName(), entry.getValue().size());
        });
        return Optional.of(summary);
    }

    private Map<String, Object> statsEvent(IItem event, JFRFileData fileData) {
        IType<IItem> itemType = ItemToolkit.getItemType(event);
        Map<String, Object> eventData = new HashMap<>();
        for (Map.Entry<IAccessorKey<?>, ? extends IDescribable> e : itemType.getAccessorKeys().entrySet()) {
            IMemberAccessor<?, IItem> accessor = itemType.getAccessor(e.getKey());
            IAccessorKey<?> key = e.getKey();
            Object value = accessor.getMember(event);
            eventData.put(key.getIdentifier(), value);
            //printValue(e.getKey(), e.getValue(), accessor.getMember(event));
        }
        fileData.getEvents().computeIfAbsent(itemType, _k -> new ArrayList<>())
                .add(eventData);
        return eventData;
    }

    public Optional<JFRFileItem> getFileItemById(String fileId) {
        return this.fileItems.stream().filter(f -> f.getId().equals(fileId)).findFirst();
    }

    private JFRFileItem createJFRFile(File file) {
        JFRFileItem fileItem = new JFRFileItem();
        fileItem.setId(getJFRFileId(file));
        fileItem.setName(file.getName());
        fileItem.setSize(file.length());
        fileItem.setPath(file.getAbsolutePath());
        //fileItem.setCreatedTime(file.);
        return fileItem;
    }

    private String getJFRFileId(File file) {
        CRC32 crc32 = new CRC32();
        crc32.update(file.getName().getBytes(StandardCharsets.UTF_8));
        return Long.toHexString(crc32.getValue());
    }
}
