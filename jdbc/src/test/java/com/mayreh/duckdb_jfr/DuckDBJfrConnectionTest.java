package com.mayreh.duckdb_jfr;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.sql.ResultSet;
import java.sql.Statement;

import org.junit.jupiter.api.Test;

public class DuckDBJfrConnectionTest {
    @Test
    void testQuery() throws Exception {
        try (DuckDBJfrConnection conn = DuckDBJfrConnection.inMemoryConnection();
             InputStream is = DuckDBJfrConnectionTest.class
                     .getClassLoader()
                     .getResourceAsStream("async-profiler-wall.jfr")) {
            Path file = Files.createTempFile("", ".jfr");
            Files.copy(is, file, StandardCopyOption.REPLACE_EXISTING);

            conn.attach(file);
            try (Statement stmt = conn.connection().createStatement()) {
                stmt.execute("select count(*) from \"jdk.ExecutionSample\"");
                ResultSet rs = stmt.getResultSet();
                rs.next();

                assertEquals(428, rs.getInt(1));
            }
        }
    }
}
