package com.mayreh.duckdb_jfr;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.Properties;

/**
 * Connection wrapper for DuckDB with JFR support.
 */
public class DuckDBJfrConnection implements AutoCloseable {
    private final Connection connection;
    private static final Path extensionPath;

    static {
        try {
            Class.forName("org.duckdb.DuckDBDriver");

            String arch = System.getProperty("os.arch").toLowerCase().trim();
            String os = System.getProperty("os.name").toLowerCase().trim();

            if (os.startsWith("mac")) {
                os = "osx";
                arch = "universal";
            } else if (os.startsWith("linux")) {
                os = "linux";
                switch (arch) {
                    case "x86_64":
                    case "amd64":
                        arch = "amd64";
                        break;
                    case "aarch64":
                    case "arm64":
                        arch = "arm64";
                        break;
                    default:
                        throw new RuntimeException("Unsupported architecture: " + arch);
                }
            } else {
                throw new RuntimeException("Unsupported OS: " + os);
            }
            String resourceName = String.format("%s-%s/libduckdb_jfr_extension.so", os, arch);
            extensionPath = extractExtension(resourceName);
        } catch (ClassNotFoundException e) {
            throw new RuntimeException(e);
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    private static Path extractExtension(String resourceName) throws IOException {
        Path tempDir = Files.createTempDirectory("duckdb_jfr");
        tempDir.toFile().deleteOnExit();

        // we need to use this filename since duckdb identifies init function by filename
        Path extensionPath = tempDir.resolve("libduckdb_jfr_extension.so");
        extensionPath.toFile().deleteOnExit();

        try (InputStream s = DuckDBJfrConnection.class.getClassLoader().getResourceAsStream(resourceName)) {
            Files.copy(s, extensionPath);
        }

        return extensionPath;
    }

    /**
     * Create a new in-memory DuckDB connection with JFR-extension preloaded.
     */
    public static DuckDBJfrConnection inMemoryConnection() throws SQLException {
        return inMemoryConnection(null);
    }

    /**
     * Create a new in-memory DuckDB connection with JFR-extension preloaded.
     */
    public static DuckDBJfrConnection inMemoryConnection(Properties properties) throws SQLException {
        Properties props = new Properties();
        if (properties != null) {
            props.putAll(properties);
        }
        props.setProperty("allow_unsigned_extensions", "true");

        Connection conn = DriverManager.getConnection("jdbc:duckdb:", props);
        try(Statement stmt = conn.createStatement()) {
            stmt.execute(String.format("load '%s'", extensionPath));
        } catch (SQLException e) {
            conn.close();
            throw e;
        }
        return new DuckDBJfrConnection(conn);
    }

    /**
     * Attach a JFR file to the connection.
     * Views for JFR events will be created.
     */
    public void attach(Path jfrFile) throws SQLException {
        try(PreparedStatement stmt = connection.prepareStatement("call jfr_attach(?)")) {
            stmt.setString(1, jfrFile.toString());
            stmt.execute();
        }
    }

    /**
     * Get underlying {@link Connection}
     */
    public Connection connection() {
        return connection;
    }

    protected DuckDBJfrConnection(Connection connection) {
        this.connection = connection;
    }

    @Override
    public void close() throws SQLException {
        connection.close();
    }
}
