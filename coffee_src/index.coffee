import m from 'mithril'

response =
  message: ''
  delay_fade: (res)->
    response.message = res
    setTimeout ()->
      response.message = ''
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
      response.delay_fade(data)
      login.get()
  get: ()->
    m.request
      method: 'get'
      url: '/user'
    .then (data)->
      response.delay_fade(data)
      counter.count = data.counter
      login.user_id_logged_in = data.user_id
  logout: ()->
    m.request
      method: 'post'
      url: '/logout'
    .then (data)->
      counter.count = 0
      login.user_id_logged_in = ''
      response.delay_fade(data)

counter =
  count: 0
  count_up: ()->
    m.request
      method: 'post'
      url: '/count_up'
    .then (data)->
      counter.count = data.counter
      response.delay_fade(data)

login_btn =
  visible: false
  pos:
    x: 0
    y: 0

PopOverBottom =
  view: (vnode)->
    x = vnode.attrs.x
    y = vnode.attrs.y
    m 'div', [
      if login_btn.visible
        m '.popover.fade.bs-popover-bottom.show',
          style:
            position: 'absolute'
            transform: "translate3d(#{x}px, #{y}px, 0px)"
            top: '0px'
            left: '0px'
            'will-change': 'transform'
        , [
            m '.arrow',
              style: left: '20px'
            m '.popover-body', 'Input Username and submit'
          ]
    ]

DisabledNavLoginButton =
  oncreate: (vnode)->
    rect = vnode.dom.getBoundingClientRect()
    login_btn.pos =
      x: rect.x-nav.pos.x
      y: rect.y+rect.height+10
  view: ()->
    m 'label.btn.btn-outline-secondary',
      onclick: ()->
        login_btn.visible = true
        setTimeout ()->
          login_btn.visible = false
        , 5000
    , 'Login'

nav =
  pos:
    x: 0
    y: 0
    height: 0
    width: 0
  collapse: true
  toggle: (e)->
    e.preventDefault()
    nav.collapse = !nav.collapse
  selected_index: 0
  select_menu: (i)->(e)->
    e.preventDefault()
    nav.selected_index = i

Nav =
  oncreate: (vnode)->
    rect = vnode.dom.getBoundingClientRect()
    nav.pos =
      x: rect.x
      y: rect.y
      height: rect.height
      width: rect.width
  view: ()->
    m 'nav.navbar.navbar-expand-md.bg-dark.navbar-dark', [
      m 'a.navbar-brand.text-light', 'Actix Redis Session'
      m 'button.navbar-toggler',
        onclick: nav.toggle
      , [m 'span.navbar-toggler-icon']
      m '.navbar-collapse',
        class: if nav.collapse then 'collapse' else 'in'
      , [
          m 'ul.navbar-nav.mr-auto', [
            name: 'Home'
          ].map (menu, i)->
            m 'li.nav-item',
              class: if nav.selected_index == i then 'active'
            , [
                m 'a.nav-link',
                  onclick: nav.select_menu(i)
                , menu.name
              ]
          if login.user_id_logged_in
            m '.form-inline', [
              m 'label.btn.btn-outline-success',
                title: 'Click to Logout'
                onclick: login.logout
              , [
                  m '.mr-2', [
                    m PowerIcon
                  ]
                  m '.text-light', 'user@domain.local'
                ]
            ]
          else
            m 'form.form-inline', [
              m 'input.form-control.mr-sm-2',
                type: 'text'
                placeholder: 'User Name'
                oninput: (e)-> login.user_id_input = e.target.value
                value: login.user_id_input
              if login.user_id_input
                m 'div', [
                  m 'input.form-control.mr-sm-2',
                    type: 'password'
                    placeholder: 'Password'
                    oninput: (e)-> login.password_input = e.target.value
                    value: login.password_input
                  m 'button.btn.btn-outline-info',
                    type: 'submit'
                    onclick: login.login
                  , 'Login'
                ]
              else
                m 'div', [
                  m DisabledNavLoginButton
                  m PopOverBottom, login_btn.pos
                ]
            ]
        ]
    ]


PowerIcon =
  view: ()->
    m.trust """
      <svg class="bi bi-power" width="1em" height="1em" viewBox="0 0 16 16" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
        <path fill-rule="evenodd" d="M5.578 4.437a5 5 0 104.922.044l.5-.866a6 6 0 11-5.908-.053l.486.875z" clip-rule="evenodd"/>
        <path fill-rule="evenodd" d="M7.5 8V1h1v7h-1z" clip-rule="evenodd"/>
      </svg>
      """

Home =
  oninit: (vnode)-> login.get()
  view: ()->
    m '.container', [
      m Nav

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
          if response.message
            m '.alert.alert-info', JSON.stringify response.message
        ]
      ]
    ]

m.mount document.getElementById('contents'), Home
