extern crate iron;
extern crate router;
extern crate rusqlite;
extern crate rustc_serialize;

use iron::prelude::*;
use std::net::*;
use iron::status;
use router::Router;
use iron::method::Method::*;
use iron::headers;
use iron::AfterMiddleware;
use rusqlite::Connection;
use rustc_serialize::json;

fn say_hello(req: &mut Request) -> IronResult<Response> {
	let message = "Hello basacdacu!";
	let mut response = Response::with((status::Ok, message));
	Ok(response)
}

#[derive(Debug)]
#[derive(RustcEncodable)]
struct Person{
	person_id: i32,
	firstname: String,
	lastname: String,
	age: i32,
	pic: String,
	status: String,
	occupation: String
}

fn get_data(req: &mut Request) -> IronResult<Response> {
	let con = Connection::open("baranggay.db");
	let page = req.extensions.get::<Router>().unwrap().find("page").unwrap();
	let page:i32 = page.parse().unwrap();
	let page_size = 5;
	let offset = page_size * page;
	match con{
		Ok(con) => {
			println!("connected!");
			let sql = format!("SELECT person_id, firstname, lastname, 
					age, pic, status, occupation
			FROM person limit {} offset {}",page_size, offset);
			let mut stmt = con.prepare(&sql).unwrap();
			let mut persons:Vec<Person> = vec![];
			let mut person_iter = stmt.query_map(&[], |row|
				{
					Person{
						person_id: row.get(0),
						firstname: row.get(1),
						lastname: row.get(2),
						age: row.get(3),
						pic: row.get(4),
						status: row.get(5),
						occupation: row.get(6)
					}
				}
			);
			match person_iter{
				Ok(person_iter) => { 
					for person in person_iter{
						let person = match person{
							Ok(person) => person,
							Err(e) => panic!("has error here"),
						};
						println!("Found person {:?}", person);
						persons.push(person);
					}
				}
				Err(e) => {
					println!("Error {:?}",e);
				}
			};
			let data = json::encode(&persons).unwrap();
			println!("data: {:#?}", data);
			let mut response = Response::with((status::Ok, data));
			Ok(response)
		},
		Err(e) => {
			println!("Unable to connect db");
			let mut response = Response::with((status::BadRequest, "Error connecting to db"));
			Ok(response)
		}
	}
}

fn search_data(req: &mut Request) -> IronResult<Response> {
	let needle = req.extensions.get::<Router>().unwrap().find("needle").unwrap();
	let con = Connection::open("baranggay.db");
	match con{
		Ok(con) => {
			println!("connected!");
			let sql = format!("SELECT person_id, firstname, lastname, 
					age, pic, status, occupation  
			FROM person 
			WHERE lastname LIKE '{}%' ",needle);
			let mut stmt = con.prepare(&sql).unwrap();
			let mut persons = vec![];
			let mut person_iter = stmt.query_map(&[], |row|
				{
					Person{
						person_id: row.get(0),
						firstname: row.get(1),
						lastname: row.get(2),
						age: row.get(3),
						pic: row.get(4),
						status: row.get(5),
						occupation: row.get(6)
					}
				}
			).unwrap();
			for person in person_iter{
				let person = person.unwrap();
				println!("Found person {:?}", person);
				persons.push(person);
			}
			let data = json::encode(&persons).unwrap();
			println!("data: {:#?}", data);
			let mut response = Response::with((status::Ok, data));
			Ok(response)
		},
		Err(e) => {
			println!("Unable to connect db");
			let mut response = Response::with((status::BadRequest, "Error connecting to db"));
			Ok(response)
		}
	}
}
fn main() {
	let mut router = Router::new();
	router.get("/", say_hello);
	router.get("/data/:page", get_data);
	router.get("/search/:needle", search_data);

	let mut middleware = Chain::new(router);
	middleware.link_after(CORS);
	let host = SocketAddrV4::new(Ipv4Addr::new(0,0,0,0), 8080);
	println!("listenning on http://{}", host);
	Iron::new(middleware).http(host).unwrap();
}

struct CORS;

impl AfterMiddleware for CORS {
	fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
		res.headers.set(headers::AccessControlAllowOrigin::Any);
        res.headers.set(headers::AccessControlAllowMethods(
                vec![Get,Head,Post,Delete,Options,Put,Patch]));
        Ok(res)		
	}}
