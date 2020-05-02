import m from 'mithril'

response = ''
delay_fade = (res)->
  response = res
  setTimeout ()->
    response = ''
  , 2000

login =
  user_id_logged_in: ''
  user_id_input: ''
  password_input: ''
  login: (e)->
    e.preventDefault()
    m.request
      method: 'post'
      url: '/login'
      body:
        user_id: login.user_id_input
        password: login.password_input
    .then (data)->
      login.user_id_input = ''
      login.password_input = ''
      delay_fade(data)
      login.get()
  get: ()->
    m.request
      method: 'get'
      url: '/user'
    .then (data)->
      delay_fade(data)
      counter.count = data.counter
      login.user_id_logged_in = data.user_id
  logout: ()->
    m.request
      method: 'post'
      url: '/logout'
    .then (data)->
      counter.count = 0
      login.user_id_logged_in = ''
      delay_fade(data)

counter =
  count: 0
  count_up: ()->
    m.request
      method: 'post'
      url: '/count_up'
    .then (data)->
      counter.count = data.counter
      delay_fade(data)


Home =
  oninit: (vnode)-> login.get()
  view: ()->
    m '.container', [
      m '.card', [
        m '.card-header', 'Actix Redis Session'
        m '.card-body', [
          if login.user_id_logged_in
            m '.row', [
              m '.col', [
                m 'h3', login.user_id_logged_in
              ]
              m '.col', [
                m 'label.btn.btn-dark',
                  onclick: login.logout
                , 'Log out'
              ]
            ]
          else
            m 'form.form-inline', [
              m 'label.mr-2', 'User ID'
              m 'input.form-control.mr-2',
                type: 'text'
                oninput: (e)-> login.user_id_input = e.target.value
                value: login.user_id_input
              m 'label.mr-2', 'Password'
              m 'input.form-control.mr-2',
                type: 'password'
                oninput: (e)-> login.password_input = e.target.value
                value: login.password_input
              m 'button.btn.btn-light',
                type: 'submit'
                onclick: login.login
              , 'Login'
            ]
          m '.row',[
            m '.col', [
              m 'h3', counter.count
            ]
            m '.col', [
              m 'label.btn.btn-light',
                onclick: counter.count_up
              , 'Count UP'
            ]
            m '.col', [
              m 'label.btn.btn-info',
                onclick: login.get
              , 'Get Current'
            ]
          ]
          if response
            m '.alert.alert-info', JSON.stringify response
        ]
      ]
    ]

m.mount document.getElementById('contents'), Home
