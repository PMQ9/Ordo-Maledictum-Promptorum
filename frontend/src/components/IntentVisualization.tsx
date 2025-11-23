import React from 'react';
import { Card } from './Card';
import { Badge } from './Badge';
import type { ParsedIntent, VotingResult, ComparisonResult } from '../types';
import { formatPercentage } from '../utils/format';

interface IntentVisualizationProps {
  parsedIntent: ParsedIntent;
  votingResult: VotingResult;
  comparisonResult: ComparisonResult;
}

export const IntentVisualization: React.FC<IntentVisualizationProps> = ({
  parsedIntent,
  votingResult,
  comparisonResult,
}) => {
  return (
    <div className="space-y-6">
      {/* Parsed Intent */}
      <Card title="Parsed Intent">
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700">Intent Type:</span>
            <span className="text-sm font-semibold text-gray-900">{parsedIntent.intent_type}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700">Confidence:</span>
            <span className="text-sm font-semibold text-gray-900">
              {formatPercentage(parsedIntent.confidence)}
            </span>
          </div>
          <div className="pt-2 border-t border-gray-200">
            <span className="text-sm font-medium text-gray-700 block mb-2">Entities:</span>
            <div className="bg-gray-50 rounded p-3">
              <pre className="text-xs text-gray-800 overflow-x-auto">
                {JSON.stringify(parsedIntent.entities, null, 2)}
              </pre>
            </div>
          </div>
          <div className="pt-2 border-t border-gray-200">
            <span className="text-sm font-medium text-gray-700 block mb-2">Original Query:</span>
            <p className="text-sm text-gray-900 italic">&quot;{parsedIntent.raw_query}&quot;</p>
          </div>
        </div>
      </Card>

      {/* Voting Results */}
      <Card title="Multi-Model Voting Results">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700">Final Decision:</span>
            <Badge status={votingResult.final_decision} />
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700">Consensus Score:</span>
            <span className="text-sm font-semibold text-gray-900">
              {formatPercentage(votingResult.consensus_score)}
            </span>
          </div>
          <div className="pt-2 border-t border-gray-200">
            <span className="text-sm font-medium text-gray-700 block mb-3">Individual Votes:</span>
            <div className="space-y-3">
              {votingResult.votes.map((vote, index) => (
                <div key={index} className="bg-gray-50 rounded-lg p-4 border border-gray-200">
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-medium text-gray-900">{vote.model_name}</span>
                    <div className="flex items-center gap-2">
                      <Badge status={vote.vote} />
                      <span className="text-xs text-gray-600">
                        ({formatPercentage(vote.confidence)})
                      </span>
                    </div>
                  </div>
                  <p className="text-sm text-gray-700">{vote.reasoning}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </Card>

      {/* Comparison Result */}
      <Card title="Intent Comparison">
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700">Match Status:</span>
            <Badge status={comparisonResult.matches ? 'allow' : 'deny'} />
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700">Similarity Score:</span>
            <span className="text-sm font-semibold text-gray-900">
              {formatPercentage(comparisonResult.similarity_score)}
            </span>
          </div>
          {comparisonResult.differences.length > 0 && (
            <div className="pt-2 border-t border-gray-200">
              <span className="text-sm font-medium text-gray-700 block mb-2">Differences:</span>
              <ul className="list-disc list-inside text-sm text-gray-700 space-y-1">
                {comparisonResult.differences.map((diff, index) => (
                  <li key={index}>{diff}</li>
                ))}
              </ul>
            </div>
          )}
          <div className="pt-2 border-t border-gray-200">
            <span className="text-sm font-medium text-gray-700 block mb-2">Analysis:</span>
            <p className="text-sm text-gray-700">{comparisonResult.reasoning}</p>
          </div>
        </div>
      </Card>
    </div>
  );
};
