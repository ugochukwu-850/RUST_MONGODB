use bson::{oid::ObjectId, Bson};
use chrono::{DateTime, TimeZone, Utc};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

//the above are your imported crates you would ned for this project

#[warn(unused_imports)]
use std::env;
use std::error::Error;
use tokio;

// and a little more

use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};

//the above would be used to connect with the remote mongo db

#[tokio::main]
pub async fn config() -> Result<Client, Box<dyn Error>> {
    // Load the MongoDB connection string from an environment variable:
    // make sure you have your environment variable set like this 
    // export MONGODB_URI="environment variable" in your terminal
    let client_uri = //init a client driver
        env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
    //println!("{:?}", client_uri); //confirm that the variable was loaded


    // A Client is needed to connect to MongoDB:
    // An extra line of code to work around a DNS issue on Windows:
    let options =
    //parse a client with the resolver config sub function
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;

    //initialize a variable to act as a cursor
    let client = Client::with_options(options)?;

    // Print the databases in our MongoDB cluster:
    println!("Databases:");
    //print all the databases in the driver using the implementation .list_databases_name
    for name in client.list_database_names(None, None).await? {
        println!("- {}", name);
    }

    //init a new collection document
    //go into the database called sample_mlfix
    //go into the collection movies
    //NOTES please make sure you load the example dataset on Atlas
    let movies = client.database("sample_mflix").collection("movies");

    //init a new doc or cluster document

    //test to show how the bson doc macro works
    // created a doc instance
    let new_doc = doc! {
       "title": "Parasite",
       "year": 2020,
       "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
       "released": Utc::now(),
    };

    //insert on result into the database
    //since the document is already in bson format you can insert it directly into the database
    let insert_result = movies.insert_one(new_doc.clone(), None).await?;
    //verify the document is inputted == the insert_one function returns an objectid of the inserted document
    println!("New document ID: {}", insert_result.inserted_id);

    // Look up one document:
    let movie: bson::Document = movies
        .find_one(
            //the find one document returns one document fufilling the given traits
            doc! {
                  "title": "Parasite"
                  //returns a document with title "parasite"
            },
            None,
        )
        .await?
        //some extra error checking
        .expect("Missing 'Parasite' document.");
    println!("Movie: {}", movie);

    // Update the document:
    //how to update a document
    let update_result = movies
        .update_one(
            doc! {
                //document to updates pk value
               "_id": &movie.get("_id")
            },
            doc! {
                //use the $set to set the particular field to update
               "$set": { "year": 2019 }
            },
            None,
        )
        .await?;
    //use the "?" to return the ok value

    //print the amount of data updated
    println!("Updated {:?} document", update_result.matched_count);

    //structures for data by using the serde
    // You use `serde` to create structs which can serialize & deserialize between BSON:

    //using the derive macro to make that struct implement the serialize and deserialize and debug trait
    #[derive(Serialize, Deserialize, Debug)]
    struct Movie {
        //if option is not create a id for the struct 
        #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
        id: Option<ObjectId>,
        title: String,
        year: i32,
        plot: String,
        // macro to convert datetime to bson_datetime
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        released: chrono::DateTime<Utc>,
    }

    impl Movie {
        //a simple way to create the Movie struct
        fn create(
            title: String,
            year: i32,
            plot: Option<String>,
            released: Option<chrono::DateTime<Utc>>,
        ) -> Movie {
            Self {
                id: None,
                title: title.to_string(),
                year: year,
                //if the plot is none set it to No single plot
                plot: plot.unwrap_or_else(|| "No single plot".to_string()),
                //and if no realease date set to now
                released: released.unwrap_or_else(|| chrono::Utc::now()),
            }
        }
    }

    // Initialize struct to be inserted:
    let captain_marvel = Movie::create("Captain Marvel".to_string(), 2002, None, None);

    // Convert `captain_marvel` to a Bson instance:
    let serialized_movie = bson::to_bson(&captain_marvel)?;
    let document = serialized_movie.as_document().unwrap();

    // Insert into the collection and extract the inserted_id value:
    let insert_result = movies.insert_one(document.to_owned(), None).await?;
    let captain_marvel_id = insert_result
        .inserted_id
        .as_object_id() //get the id as an objet id without this you would not get the real _id 
        .expect("Retrieved _id should have been of type ObjectId"); //some extra erro checking
    //print the new inserted objects id
    println!("Captain Marvel document ID: {:?}", captain_marvel_id);

    // Look up one document:
    let movie: bson::Document = movies
        .find_one(
            doc! {
                  "_id": captain_marvel_id
            },
            None,
        )
        .await?
        .expect("Missing 'Parasite' document.");
    println!("Movie: {}", movie);

    // Retrieve Captain Marvel from the database, into a Movie struct:
    // Read the document from the movies collection:
    //just another way to find one document 
    let loaded_movie = movies
        .find_one(Some(doc! { "_id":  captain_marvel_id.clone() }), None)
        .await?
        .expect("Document not found");

    // Deserialize the document into a Movie instance
    let loaded_movie_struct: Movie = bson::from_bson(Bson::Document(loaded_movie))?;
    println!("Movie loaded from collection: {:?}", loaded_movie_struct);

    let deleted_movie = movies.find_one_and_delete(
        doc!{
            "_id": captain_marvel_id
    }, None).await?.expect("Id should be in database already:: deleted");

    print!("\n\n THE MOVIE HAS BEEN DELETED");
    println!("{:?}",deleted_movie);

    Ok(client)
}


//FOLLOW FOR MORE