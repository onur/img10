# img10

Temporary image hosting service based on Google App Engine and Google Cloud
Storage.

## Sharing Screenshots Temporarily

You can use `./screenshot` script to take screenshots with scrot and upload
them to img10.xyz.

![screenshot sharing](https://i.imgur.com/MhuGxwj.png "screenshot sharing")

## Installation

Clone repository, install requirements with pip and download
[Google App Engine SDK](https://cloud.google.com/appengine/downloads#Google_App_Engine_SDK_for_Python)
and start the `dev_appserver.py` in repository directory.

```sh
git clone https://github.com/onur/img10.git img10
cd img10
pip install -t lib -r requirements.txt
$PATH_TO_APPENGINE_SDK/dev_appserver.py .
```

Your instance should be available in: `http://localhost:8080`


## TODO

- [ ] Drag and drop support.
