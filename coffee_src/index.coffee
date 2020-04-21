import m from 'mithril'

user_json = {}
get_user = ()->
  m.request
    method: 'get'
    url: '/user'
  .then (data)->
    user_json = data

login_json = {}
post_login = ()->
  m.request
    method: 'post'
    url: '/login'
    body: user_id: 'user1'
  .then (data)->
     login_json = data

logout_json = {}
post_logout = ()->
  m.request
    method: 'post'
    url: '/logout'
  .then (data)->
     logout_json = data

do_something_json = {}
post_count_up = ()->
  m.request
    method: 'post'
    url: '/count_up'
  .then (data)->
     do_something_json = data

Home =
  view: ()->
    m '.container', [
      m '.panel.panel-default', [
        m '.panel-heading', 'Actix Redis Session'
        m '.panel-body', [
          m 'label.btn.btn-default',
            onclick: get_user
          , 'Get User'
          m 'label.form-control', JSON.stringify user_json
          m 'label.btn.btn-default',
            onclick: post_login
          , 'Login'
          m 'label.form-control', JSON.stringify login_json
          m 'label.btn.btn-default',
            onclick: post_count_up
          , 'count up'
          m 'label.form-control', JSON.stringify do_something_json
          m 'label.btn.btn-default',
            onclick: post_logout
          , 'Logout'
          m 'label.form-control', JSON.stringify logout_json
        ]
      ]
    ]

m.mount document.getElementById('contents'), Home
