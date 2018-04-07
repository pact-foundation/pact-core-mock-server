require 'pact_mockserver_mk2'

RSpec.describe Pact::Mockserver::Mk2 do
  it 'has a version number' do
    expect(Pact::Mockserver::Mk2::VERSION).not_to be nil
  end
end
