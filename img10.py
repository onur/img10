#!/usr/bin/env python
# Copyright (C) <2016>  Onur Aslan  <onur@onur.im>
# See COPYING for distribution information.

import os
from random import randint
from google.appengine.api import images
from google.appengine.api import app_identity
from google.appengine.ext import ndb
from google.appengine.ext import vendor
import webapp2
import jinja2
import datetime

vendor.add(os.path.join(os.path.dirname(os.path.realpath(__file__)), 'lib'))
import cloudstorage as gcs


# Images removed after timeout (in seconds)
TIMEOUT = 1800

JINJA_ENVIRONMENT = jinja2.Environment(
    loader=jinja2.FileSystemLoader(os.path.dirname(__file__)),
    extensions=['jinja2.ext.autoescape'],
    autoescape=True)

TEMPLATE = JINJA_ENVIRONMENT.get_template('img10.html')

BUCKET_NAME = os.environ.get('BUCKET_NAME',
                             app_identity.get_default_gcs_bucket_name())


ACME_VERIFICATION = ""


def generate_id(size=7):
    chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
    id = list()
    for i in range(size):
        id.append(chars[randint(0, len(chars) - 1)])
    return "".join(id)


def remove_image(image_id):
    try:
        gcs.delete('/' + BUCKET_NAME + '/' + image_id)
    except:
        pass


class Images(ndb.Model):
    """ Images model """
    mime = ndb.StringProperty(required=True)
    date = ndb.DateTimeProperty(auto_now_add=True)
    thumbnail = ndb.BlobProperty(required=True)


class Image(webapp2.RequestHandler):
    """ Image handler to serve images """
    def get(self, image_id, extension):
        image = ndb.Key(Images, image_id).get()

        if image:
            delta = (datetime.datetime.now() - image.date).total_seconds()
            if delta >= TIMEOUT:
                self.not_found()
                return

            gcs_file = gcs.open('/' + BUCKET_NAME + '/' + image_id)
            content = gcs_file.read()
            gcs_file.close()

            if extension == 'jpg' and image.mime == 'image/png':
                orig_image = images.Image(content)
                orig_image.rotate(0)
                self.response.headers['Content-Type'] = 'image/jpeg'
                self.response.write(
                        orig_image.execute_transforms(
                            output_encoding=images.JPEG, quality=90))
            else:
                self.response.headers['Content-Type'] = str(image.mime)
                self.response.write(content)
            return

        self.not_found()

    def not_found(self):
        self.response.set_status(404)
        self.response.write(TEMPLATE.render({
            'error': 'File not found!'
        }))


class Thumbnail(webapp2.RequestHandler):
    """ Image handler to serve images """
    def get(self, image_id):
        image = ndb.Key(Images, image_id).get()

        if image:
            self.response.headers['Content-Type'] = 'image/jpeg'
            self.response.write(image.thumbnail)
            return

        self.error(404)


class Main(webapp2.RequestHandler):
    def get(self):
        self.response.write(TEMPLATE.render())


class RemoveOldImages(webapp2.RequestHandler):
    def get(self):
        images = Images.query(
                Images.date <= (datetime.datetime.now() -
                                datetime.timedelta(seconds=TIMEOUT)))
        for image in images:
            remove_image(image.key.id())
            image.key.delete()


class Upload(webapp2.RequestHandler):
    """ Upload handler """
    def post(self):
        img_from_request = self.request.get('img')
        image = images.Image(img_from_request)
        mime = ""
        extension = ""
        try:
            if image.format == images.JPEG:
                mime = "image/jpeg"
                extension = ".jpg"
            elif image.format == images.PNG:
                mime = "image/png"
                extension = ".png"
        except:
            self.response.set_status(500)
            self.response.write(TEMPLATE.render({
                'error': 'Unrecognized image format!'
            }))
            return

        id = generate_id()
        while ndb.Key(Images, id).get():
            id = generate_id()

        write_retry_params = gcs.RetryParams(backoff_factor=1.1)
        gcs_file = gcs.open('/' + BUCKET_NAME + '/' + id,
                            'w',
                            content_type=mime,
                            retry_params=write_retry_params)
        gcs_file.write(img_from_request)
        gcs_file.close()

        thumb_img = images.Image(img_from_request)
        thumb_img.resize(width=492)
        thumbnail = thumb_img.execute_transforms(output_encoding=images.JPEG)

        img = Images(id=id,
                     mime=mime,
                     thumbnail=thumbnail)
        img_key = img.put()
        img_url = self.request.host_url + '/' + id + extension

        if self.request.headers.get('User-Agent').find('curl') >= 0:
            self.response.headers['Content-Type'] = 'text/plain'
            self.response.write(img_url + "\n")
        else:
            self.response.write(TEMPLATE.render({
                'img_id': img_key.id(),
                'img_url': self.request.host_url + '/' + id + extension
            }))


class LetsEncrypt(webapp2.RequestHandler):
    """ LetsEncrypt handler to verify ACME requests """
    def get(self):
        self.response.headers['Content-Type'] = 'text/plain'
        self.response.write(ACME_VERIFICATION)


app = webapp2.WSGIApplication([
    ('/', Main),
    (r'/(\w+)\.(jpg|png)', Image),
    (r'/t/(\w+)\.jpg', Thumbnail),
    ('/upload', Upload),
    ('/tasks/remove', RemoveOldImages),
    ('/.well-known/acme-challenge/.*?', LetsEncrypt),
], debug=True)
