CREATE TABLE files (
	ino INTEGER PRIMARY KEY,
	parent INTEGER NOT NULL,
	type INTEGER NOT NULL,
	name BLOB,
	folded BLOB,
	rc INTEGER NOT NULL DEFAULT 0,
	UNIQUE (parent, folded),
	FOREIGN KEY (parent)
		REFERENCES files (ino)
			ON DELETE CASCADE
			ON UPDATE CASCADE
);
CREATE TABLE opendir (
	fh INTEGER PRIMARY KEY,
	ino INTEGER NOT NULL,
	FOREIGN KEY (ino)
		REFERENCES files (ino)
			ON DELETE CASCADE
			ON UPDATE CASCADE
);
CREATE TABLE readdir (
	fh INTEGER NOT NULL,
	ino INTEGER NOT NULL,
	name BLOB NOT NULL,
	type INTEGER NOT NULL,
	UNIQUE (fh, ino)
		ON CONFLICT REPLACE,
	FOREIGN KEY (fh)
		REFERENCES opendir (fh)
			ON DELETE CASCADE
			ON UPDATE CASCADE,
	FOREIGN KEY (ino)
		REFERENCES files (ino)
			ON DELETE CASCADE
			ON UPDATE CASCADE
);
CREATE TABLE paths_to_delete (
	name BLOB NOT NULL
);
CREATE TRIGGER delete_file
	AFTER UPDATE
	ON files
	WHEN new.rc = 0 AND new.parent = 0
BEGIN
	INSERT INTO paths_to_delete (name) VALUES (new.name);
	DELETE FROM files
	WHERE ino = new.ino;
END;
INSERT INTO files (ino, parent, name, folded, rc, type) 
	VALUES (0, 0, NULL, NULL, 1, 24576);
INSERT INTO files (ino, parent, name, folded, rc,  type) 
	VALUES (1, 0, "", "", 1, 16384);
