require 'httparty'

RSpec.describe 'Simple Consumer Spec' do

  let(:pact) do
    '
      {
        "provider": {
          "name": "Alice Service"
        },
        "consumer": {
          "name": "Consumer"
        },
        "interactions": [
          {
            "description": "a retrieve Mallory request",
            "request": {
              "method": "GET",
              "path": "/mallory",
              "query": "name=ron&status=good"
            },
            "response": {
              "status": 200,
              "headers": {
                "Content-Type": "text/html"
              },
              "body": "That is some good Mallory."
            }
          }
        ],
        "metadata": {
          "pact-specification": {
            "version": "1.0.0"
          },
          "pact-jvm": {
            "version": "1.0.0"
          }
        }
      }
    '
  end

  let(:mock_server_port) { PactMockServerMk2::create_mock_server(pact, 0) }

  after do
    PactMockServerMk2::cleanup_mock_server(mock_server_port)
  end

  it 'executes the pact test with no errors' do
    puts "Mock server port=#{mock_server_port}"

    response = HTTParty.get("http://localhost:#{mock_server_port}/mallory?name=ron&status=good")

    expect(response.body).to eq 'That is some good Mallory.'
    expect(PactMockServerMk2::all_matched(mock_server_port)).to be true
  end
end
