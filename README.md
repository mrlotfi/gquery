# GQuery
An in-memory database for geographic/geometric queries with experimental support for persistence. Currently the only way to interact with it is via it's HTTP REST apis. 
## Getting started
#### Docker
To run the latest version in a docker container:
```shell script
docker pull mrlotfi/gquery
docker run -p 6985:6985 mrlotfi/gquery
```
#### Manual build
Get a working rust environment ([rustup is recommended](https://rustup.rs/)). It should be compilable on most
supported platforms as it has no platform specific dependency. To build an optimized version run
```cargo build --release```. Binary will be accessible on ```./target/release/gquery```. 
#### Binary releases
To be announced.
## Running
Just run `gquery`. No additional parameter is required for running with default configuration. But you can take a look at `gquery --help`.
## Storage API
Items are stored in different namespaced collections. Each item has a unique id which is provided by you or automatically selected and returned by server.
####Adding a new item
Path:
```
POST /{collection_name}
```
Request body example:
```json
{
  "id": "abc",
  "geojson": {
    "coordinates": [
      [
        [-1.0, 1.0],
        [0.0, -1.0],
        [1.0, 1.0]
      ]
    ],
    "properties": {
      "name": "hassan"
    },
    "type": "MultiLineString"
  }
}
```
Response:
Id of the added item is returned as plain text. It is either provided by you or automatically generated by the server.

####Deleting an item
Path:
```
DELETE /{collection_name}/{id}/
```
such as ```http://localhost:6985/abc/efg/```

####Dropping an entire collection
Path:
```
DELETE /{collection_name}/
```
such as ```http://localhost:6985/abc```

####Retrieving an item by id
Path:
```
GET /{collection_name}/{id}/
```
such as ```http://localhost:6985/abc/efg/```

Reponse body example:
```json
{
  "id": "123123",
  "geojson": {
    "coordinates": [
      [
        [-1.0, 1.0],
        [0.0, -1.0],
        [1.0, 1.0]
      ]
    ],
    "properties": {
      "name": "hassan"
    },
    "type": "MultiLineString"
  }
}
```
####Retrieving list of all collections
Path:
```
GET /
```

Response body example:
```json
[
  "abc",
  "my_awesome_collection"
]
```

####(Experimental) Persisting the current snapshot
Path:
```
PUT /save
```
Returns 200 in case of succession.

## Query API
####Retrieving the nearest geometry
Path:
```
GET /{collection_name}/nearby?long=-0.1&lat=0.122
```
Query Params:
   * long (x) of desired query point
   * lat (y) of desired query point
Returns `404` if collection doesn't exist or empty.

Response body example:
```json
{
  "id": "abc",
  "geojson": {
    "coordinates": [
      [
        [-1.0, 1.0],
        [0.0, -1.0],
        [1.0, 1.0]
      ]
    ],
    "properties": {
      "name": "hassan"
    },
    "type": "MultiLineString"
  }
}
```

####Retrieving the intersecting geometry
Path:
```
GET /{collection_name}/intersect?long=-0.1&lat=0.122
```
Query Params:
   * long (x) of desired query point
   * lat (y) of desired query point
Returns `404` if collection doesn't exist or empty or no object containing this point is found.

Response body example:
```json
{
  "id": "hiva",
  "geojson": {
    "coordinates": [
      [
        [-1.0, -1.0],
        [-1.0, 1.0],
        [1.0, 1.0],
        [1.0, -1.0],
        [-1.0, -1.0]
      ]
    ],
    "properties": {
      "name": "hassan"
    },
    "type": "Polygon"
  }
}
```