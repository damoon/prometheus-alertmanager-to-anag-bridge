disable_snapshots()
allow_k8s_contexts(['test', 'ci'])

k8s_yaml('deployment/prometheus.yaml')
k8s_yaml('deployment/alertmanager.yaml')
k8s_yaml('deployment/bridge.yaml')

k8s_resource(
  'alertmanager',
  port_forwards=['9093'],
)
k8s_resource(
  'prometheus',
  port_forwards=['9090'],
)
k8s_resource(
  'bridge',
  port_forwards=['0.0.0.0:8080:8080'],
  links=['http://127.0.0.1:8080'],
)

target='prod'
live_update=[]
if os.environ.get('PROD', '') ==  '':
  target='dev'
  live_update=[
    sync('Cargo.toml', '/app/Cargo.toml'),
    sync('Cargo.lock', '/app/Cargo.lock'),
    sync('src',        '/app/src'),
    sync('patam.toml', '/app/patam.toml'),
  ]

docker_build(
  'patam-image',
  '.',
  dockerfile='deployment/Dockerfile',
  target=target,
  build_args={"SOURCE_BRANCH":"development", "SOURCE_COMMIT":"development"},
  only=[ 'Cargo.toml'
       , 'Cargo.lock'
       , 'src'
       , 'patam.toml'
  ],
  live_update=live_update,
)
