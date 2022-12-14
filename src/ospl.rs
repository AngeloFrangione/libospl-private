/*	libospl - Open Source Photo Library
	an opensource and multiplateform photo library management that can be used
	to store and sort all your photos.
	Copyright (C) 2019-2022 Angelo Frangione

	This program is free software; you can redistribute it and/or modify
	it under the terms of the GNU General Public License as published by
	the Free Software Foundation; either version 2 of the License, or
	(at your option) any later version.

	This program is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU General Public License for more details.

	You should have received a copy of the GNU General Public License along
	with this program; if not, write to the Free Software Foundation, Inc.,
	51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
*/

static DATABASE_FILENAME: &str = "database.db";
static LIBRARY_EXTENSION: &str = ".ospl";

static VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
static VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
static VERSION_REVISION: &str = env!("CARGO_PKG_VERSION_PATCH");

mod database;
mod directory;

use database::Database;
use directory::Directory;

#[derive(Debug)]
pub enum Error
{
	Other = -1000,		// other error
	Exists,				// the file or folder already exists
	DB,					// database communication failed
	NotFound,			// element not found
	PermissionDenied,	// no permission to create or read file
	NotSupported,		// element not supported
	Thumb,				// thumbnail creation failed
	PhoNF,				// photo not found in db
	AlbNF,				// album not found in db
}

pub struct Library
{
	path: String,
	db: Database,
}

impl Library
{
	pub fn create(path: &String) -> Result<Self, Error>
	{
		match Directory::from(&path)?.create()
		{
			Ok(_) =>
			{
				Ok(Library
				{
					path: path.clone(),
					db: Database::create(&path)?
				})
			},
			Err(e) => Err(e),
		}
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	use rand::{thread_rng, Rng};
	use rand::distributions::Alphanumeric;

	use rusqlite::{Connection};

	static TEST_DIR: &str = "/tmp/";
	static LIBRARY_CREATE_ERROR: &str = "error creating library";

	fn remove_test_path(path: String)
	{
		println!("removing test dir");
		match std::fs::remove_dir_all(path)
		{
			Ok(_) => {},
			Err(e) => {println!("{:?}", e)}
		}
	}

	fn generate_test_path() -> String
	{
		let rand_string: String = thread_rng()
			.sample_iter(&Alphanumeric)
			.take(30)
			.map(char::from)
			.collect();
		TEST_DIR.to_string() + &rand_string + &LIBRARY_EXTENSION.to_string()
	}

	fn check_table_presence(name: &str, db: &str) -> bool
	{
		let conn = Connection::open(db).unwrap();
		let mut conn = conn.prepare(
			"SELECT name FROM sqlite_master WHERE type='table' AND name=?")
			.unwrap();
		let mut table: String = "".to_string();
		let mut rows = conn.query([name]).unwrap();
		while let Some(row) = rows.next().unwrap()
		{
			table = row.get(0).unwrap();
		}
		println!("check if table {} is present: table found: {}", name, table);
		name.eq(&table)
	}

	#[test]
	fn library_path()
	{
		let path = generate_test_path();
		let _library = match Library::create(&path)
		{
			Ok(lib) =>
			{
				println!("check if {} == {}", lib.path, path);
				assert_eq!(lib.path, path);
			},
			Err(e) => {panic!("{}: {:?}", LIBRARY_CREATE_ERROR, e)},
		};
		remove_test_path(path);
	}
	#[test]
	fn library_database()
	{
		let path = generate_test_path();
		let db_path = path.clone() + "/database.db";

		let _library = match Library::create(&path)
		{
			Ok(_lib) =>
			{
				println!("checking if database has been created at {}", &db_path);
				assert!(std::path::Path::new(&db_path).exists());
			},
			Err(e) => panic!("{}: {:?}", LIBRARY_CREATE_ERROR, e),
		};
		assert!(check_table_presence("settings", &db_path));
		assert!(check_table_presence("photos", &db_path));
		assert!(check_table_presence("includes", &db_path));
		assert!(check_table_presence("holds", &db_path));
		assert!(check_table_presence("contains", &db_path));
		assert!(check_table_presence("albums", &db_path));
		remove_test_path(path);
	}

	#[test]
	#[should_panic]
	fn create_library_no_permissions()
	{
		let _library = match Library::create(&"/root/library".to_string())
		{
			Ok(_) => println!("trying to create library at path '/root/library' should return permission denied"),
			Err(e) =>
			{
				match e
				{
					Error::PermissionDenied => panic!("ok: could not create library at path '/root/library': permission denied"),
					_ => println!("error: unexpected error returned, but shouldn't have: {:?}", e),
				}
			}
		};
	}

	#[test]
	#[should_panic]
	fn create_library_exists()
	{
		let _library = match Library::create(&"/".to_string())
		{
			Ok(_) => println!("trying to create library at path '/' should return path already exists"),
			Err(e) =>
			{
				match e
				{
					Error::Exists => panic!("ok: could not create library at path '/': folder exists"),
					_ => println!("error: unexpected error returned, but shouldn't have: {:?}", e),
				}
			}
		};
	}
}

