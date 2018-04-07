require 'httparty'

RSpec.describe 'Simple Consumer Spec' do

  describe 'with matching requests' do

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


  describe 'with mismatching requests' do

    let(:pact) do
      '
      {
        "provider": {
          "name": "test_provider"
        },
        "consumer": {
          "name": "test_consumer"
        },
        "interactions": [
          {
            "providerState": "test state",
            "description": "test interaction",
            "request": {
              "method": "POST",
              "path": "/",
              "body": {
                "complete": {
                  "certificateUri": "http://...",
                  "issues": {
                    "idNotFound": {}
                  },
                  "nevdis": {
                    "body": null,
                    "colour": null,
                    "engine": null
                  },
                  "body": 123456
                },
                "body": [
                  1,
                  2,
                  3
                ]
              }
            },
            "response": {
              "status": 200
            }
          }
        ],
        "metadata": {
          "pact-specification": {
            "version": "2.0.0"
          },
          "pact-jvm": {
            "version": ""
          }
        }
      }
      '
    end

    let(:mock_server_port) { PactMockServerMk2::create_mock_server(pact, 0) }

    after do
      PactMockServerMk2::cleanup_mock_server(mock_server_port)
    end

    it 'returns the mismatches' do
      puts "Mock server port=#{mock_server_port}"

      expect(PactMockServerMk2::all_matched(mock_server_port)).to be false

      response1 = HTTParty.post("http://localhost:#{mock_server_port}/",
                                :headers => {'Content-Type': 'application/json'}, :body => '{}')

      response2 = HTTParty.put("http://localhost:#{mock_server_port}/mallory", body: {
        :complete => {
          :certificateUri => "http://...",
          :issues => {},
          :nevdis => {
            :body => "red",
            :colour => nil,
            :engine => nil
          },
          :body => "123456"
        },
        :body => [1, 3]
      })

      expect(PactMockServerMk2::all_matched(mock_server_port)).to be false
      mismatchers = PactMockServerMk2::mock_server_mismatches(mock_server_port)
      puts mismatchers
      expect(mismatchers.length).to eql(2)
    end
  end

end
