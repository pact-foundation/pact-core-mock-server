require 'helix_runtime'

begin
  require 'pact_mockserver_mk2/native'
rescue LoadError => e
  warn 'Unable to load pact_mockserver_mk2 native. Please run `rake build`'
  warn e
end

require 'pact/mockserver/mk2'
